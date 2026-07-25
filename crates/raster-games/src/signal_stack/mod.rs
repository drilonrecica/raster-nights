// SPDX-License-Identifier: MPL-2.0

//! Deterministic revision-1 rules for Signal Stack Standard Transmission.

mod render;
mod tables;

pub use render::render;

use rand_chacha::ChaCha8Rng;
use rand_core::{Rng, SeedableRng};
use raster_engine::{
    GameAction, GameId, GameOutcome, GameResult, GameStatus, ModeId, RulesRevision, RunSeed,
    SimulationStep, SimulationTick, StateHash,
};

pub const MATRIX_WIDTH: i8 = 10;
pub const MATRIX_HEIGHT: i8 = 24;
pub const HIDDEN_ROWS: i8 = 4;
pub const VISIBLE_ROWS: i8 = 20;
pub const PREVIEW_COUNT: usize = 5;

const MATRIX_CELLS: usize = MATRIX_WIDTH as usize * MATRIX_HEIGHT as usize;
const MAX_LOCK_RESETS: u8 = 15;

#[must_use]
pub fn rules_revision() -> RulesRevision {
    RulesRevision::new(1).expect("revision one is nonzero")
}

/// One of the seven packet geometries.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum Packet {
    I = 1,
    J = 2,
    L = 3,
    O = 4,
    S = 5,
    T = 6,
    Z = 7,
}

/// Packet orientation in clockwise order.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
#[repr(u8)]
pub enum Rotation {
    #[default]
    Zero = 0,
    Right = 1,
    Two = 2,
    Left = 3,
}

impl Rotation {
    const fn clockwise(self) -> Self {
        match self {
            Self::Zero => Self::Right,
            Self::Right => Self::Two,
            Self::Two => Self::Left,
            Self::Left => Self::Zero,
        }
    }

    const fn counterclockwise(self) -> Self {
        match self {
            Self::Zero => Self::Left,
            Self::Right => Self::Zero,
            Self::Two => Self::Right,
            Self::Left => Self::Two,
        }
    }
}

/// Integer matrix coordinate.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct CellPoint {
    pub x: i8,
    pub y: i8,
}

impl CellPoint {
    pub const fn new(x: i8, y: i8) -> Self {
        Self { x, y }
    }
}

/// The currently falling packet. `x` and `y` offset its revision-1 table.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ActivePacket {
    pub packet: Packet,
    pub rotation: Rotation,
    pub x: i8,
    pub y: i8,
}

impl ActivePacket {
    #[must_use]
    pub fn cells(self) -> [CellPoint; 4] {
        tables::cells(self.packet, self.rotation).map(|cell| CellPoint {
            x: cell.x + self.x,
            y: cell.y + self.y,
        })
    }
}

/// Signal Stack lifecycle state. Pausing is controlled by the application.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum SignalStackStatus {
    Running = 1,
    Paused = 2,
    Saturated = 3,
}

/// Last successful maneuver relevant to phase-rotation detection.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum LastManeuver {
    None = 0,
    Lateral = 1,
    Rotation = 2,
}

/// Authoritative Signal Stack simulation.
#[derive(Clone, Debug)]
pub struct SignalStack {
    seed: RunSeed,
    status: SignalStackStatus,
    tick: SimulationTick,
    matrix: [Option<Packet>; MATRIX_CELLS],
    active: Option<ActivePacket>,
    hold: Option<Packet>,
    hold_available: bool,
    previews: [Packet; PREVIEW_COUNT],
    bag: [Packet; 7],
    bag_cursor: u8,
    bag_ordinal: u64,
    score: u64,
    cleared_channels: u32,
    rate: u32,
    gravity_counter: u32,
    lock_timer: u8,
    lock_resets: u8,
    last_maneuver: LastManeuver,
    signal_chain: Option<u32>,
    sustained_transmission: bool,
    pending_spawn: bool,
    queued_actions: Vec<GameAction>,
}

impl SignalStack {
    #[must_use]
    pub fn new(seed: RunSeed) -> Self {
        let bag = shuffled_bag(seed, 0);
        let mut game = Self {
            seed,
            status: SignalStackStatus::Running,
            tick: SimulationTick(0),
            matrix: [None; MATRIX_CELLS],
            active: None,
            hold: None,
            hold_available: true,
            previews: [Packet::I; PREVIEW_COUNT],
            bag,
            bag_cursor: 0,
            bag_ordinal: 0,
            score: 0,
            cleared_channels: 0,
            rate: 1,
            gravity_counter: 0,
            lock_timer: 0,
            lock_resets: 0,
            last_maneuver: LastManeuver::None,
            signal_chain: None,
            sustained_transmission: false,
            pending_spawn: false,
            queued_actions: Vec::with_capacity(4),
        };
        let first = game.take_from_bag();
        for index in 0..PREVIEW_COUNT {
            game.previews[index] = game.take_from_bag();
        }
        game.spawn(first);
        game
    }

    pub fn reset(&mut self, seed: RunSeed) {
        *self = Self::new(seed);
    }

    /// Queues a normalized action for the next authoritative update.
    pub fn handle_action(&mut self, action: GameAction) {
        if matches!(self.status, SignalStackStatus::Running) {
            self.queued_actions.push(action);
        }
    }

    /// Advances one fixed simulation tick.
    pub fn update(&mut self, _step: SimulationStep) {
        if !matches!(self.status, SignalStackStatus::Running) {
            self.queued_actions.clear();
            return;
        }
        self.tick = SimulationTick(self.tick.0.saturating_add(1));

        if self.pending_spawn {
            self.pending_spawn = false;
            let packet = self.shift_preview();
            self.spawn(packet);
            if !matches!(self.status, SignalStackStatus::Running) {
                self.queued_actions.clear();
                return;
            }
        }

        let actions = std::mem::take(&mut self.queued_actions);
        for action in actions {
            if self.active.is_none() {
                break;
            }
            self.apply_action(action);
        }
        self.queued_actions = Vec::with_capacity(4);

        if self.active.is_none() {
            return;
        }

        self.gravity_counter = self.gravity_counter.saturating_add(1);
        if self.gravity_counter >= gravity_interval(self.rate) {
            self.gravity_counter = 0;
            self.try_translate(0, 1, false);
        }

        if self.is_grounded() {
            self.lock_timer = self.lock_timer.saturating_add(1);
            if self.lock_timer >= lock_delay(self.rate) {
                self.lock_active();
            }
        }
    }

    pub fn set_paused(&mut self, paused: bool) {
        match (self.status, paused) {
            (SignalStackStatus::Running, true) => {
                self.status = SignalStackStatus::Paused;
                self.queued_actions.clear();
            }
            (SignalStackStatus::Paused, false) => self.status = SignalStackStatus::Running,
            _ => {}
        }
    }

    #[must_use]
    pub const fn status(&self) -> SignalStackStatus {
        self.status
    }

    #[must_use]
    pub const fn tick(&self) -> SimulationTick {
        self.tick
    }

    #[must_use]
    pub const fn game_status(&self) -> GameStatus {
        match self.status {
            SignalStackStatus::Running | SignalStackStatus::Paused => GameStatus::Running,
            SignalStackStatus::Saturated => GameStatus::Finished,
        }
    }

    #[must_use]
    pub fn result(&self) -> Option<GameResult> {
        matches!(self.status, SignalStackStatus::Saturated).then(|| GameResult {
            game_id: GameId::parse("signal-stack").expect("static game identifier is valid"),
            mode_id: ModeId::parse("standard-transmission")
                .expect("static mode identifier is valid"),
            rules_revision: rules_revision(),
            seed: self.seed,
            final_tick: self.tick,
            score: self.score,
            outcome: GameOutcome::GameOver,
            final_state_hash: self.state_hash(),
        })
    }

    #[must_use]
    pub const fn matrix(&self) -> &[Option<Packet>; MATRIX_CELLS] {
        &self.matrix
    }

    #[must_use]
    pub fn cell(&self, x: i8, y: i8) -> Option<Packet> {
        matrix_index(x, y).and_then(|index| self.matrix[index])
    }

    #[must_use]
    pub const fn active(&self) -> Option<ActivePacket> {
        self.active
    }

    #[must_use]
    pub const fn hold(&self) -> Option<Packet> {
        self.hold
    }

    #[must_use]
    pub const fn hold_available(&self) -> bool {
        self.hold_available
    }

    #[must_use]
    pub const fn previews(&self) -> &[Packet; PREVIEW_COUNT] {
        &self.previews
    }

    #[must_use]
    pub const fn score(&self) -> u64 {
        self.score
    }

    #[must_use]
    pub const fn cleared_channels(&self) -> u32 {
        self.cleared_channels
    }

    #[must_use]
    pub const fn rate(&self) -> u32 {
        self.rate
    }

    #[must_use]
    pub const fn lock_timer(&self) -> u8 {
        self.lock_timer
    }

    #[must_use]
    pub const fn lock_resets(&self) -> u8 {
        self.lock_resets
    }

    #[must_use]
    pub const fn pending_spawn(&self) -> bool {
        self.pending_spawn
    }

    #[must_use]
    pub const fn last_maneuver(&self) -> LastManeuver {
        self.last_maneuver
    }

    #[must_use]
    pub const fn signal_chain_index(&self) -> Option<u32> {
        self.signal_chain
    }

    #[must_use]
    pub const fn sustained_transmission(&self) -> bool {
        self.sustained_transmission
    }

    #[must_use]
    pub fn state_hash(&self) -> StateHash {
        let mut hash = Fnv1a::new();
        hash.u16(rules_revision().get());
        hash.u8(self.status as u8);
        hash.u64(self.seed.0);
        for cell in self.matrix {
            hash.u8(cell.map_or(0, |packet| packet as u8));
        }
        hash.option(self.active, |hash, active| {
            hash.u8(active.packet as u8);
            hash.u8(active.rotation as u8);
            hash.i8(active.x);
            hash.i8(active.y);
        });
        hash.option(self.hold, |hash, packet| hash.u8(packet as u8));
        hash.bool(self.hold_available);
        hash.u32(PREVIEW_COUNT as u32);
        for packet in self.previews {
            hash.u8(packet as u8);
        }
        hash.u32(self.bag.len() as u32);
        for packet in self.bag {
            hash.u8(packet as u8);
        }
        hash.u8(self.bag_cursor);
        hash.u64(self.bag_ordinal);
        hash.u64(self.score);
        hash.u32(self.cleared_channels);
        hash.u32(self.rate);
        hash.u32(self.gravity_counter);
        hash.u8(self.lock_timer);
        hash.u8(self.lock_resets);
        hash.u8(self.last_maneuver as u8);
        hash.option(self.signal_chain, |hash, chain| hash.u32(chain));
        hash.bool(self.sustained_transmission);
        hash.bool(self.pending_spawn);
        StateHash(hash.finish())
    }

    fn apply_action(&mut self, action: GameAction) {
        match action {
            GameAction::MoveLeft => {
                self.try_translate(-1, 0, true);
            }
            GameAction::MoveRight => {
                self.try_translate(1, 0, true);
            }
            GameAction::MoveDown | GameAction::SoftDrop => {
                if self.try_translate(0, 1, false) {
                    self.score = self.score.saturating_add(1);
                }
            }
            GameAction::HardDrop => self.hard_drop(),
            GameAction::RotateClockwise => self.try_rotate(true),
            GameAction::RotateCounterclockwise => self.try_rotate(false),
            GameAction::Hold => self.try_hold(),
            GameAction::MoveUp | GameAction::Primary | GameAction::Secondary => {}
        }
    }

    fn try_translate(&mut self, dx: i8, dy: i8, lateral: bool) -> bool {
        let Some(active) = self.active else {
            return false;
        };
        let was_grounded = self.is_grounded();
        let candidate = ActivePacket {
            x: active.x.saturating_add(dx),
            y: active.y.saturating_add(dy),
            ..active
        };
        if self.collides(candidate) {
            return false;
        }
        self.active = Some(candidate);
        if lateral {
            self.last_maneuver = LastManeuver::Lateral;
            self.maybe_refresh_lock(was_grounded);
        }
        true
    }

    fn try_rotate(&mut self, clockwise: bool) {
        let Some(active) = self.active else {
            return;
        };
        if matches!(active.packet, Packet::O) {
            return;
        }
        let was_grounded = self.is_grounded();
        let next_rotation = if clockwise {
            active.rotation.clockwise()
        } else {
            active.rotation.counterclockwise()
        };
        for &(dx, dy) in tables::kicks(active.packet, active.rotation, next_rotation) {
            let candidate = ActivePacket {
                rotation: next_rotation,
                x: active.x.saturating_add(dx),
                y: active.y.saturating_add(dy),
                ..active
            };
            if !self.collides(candidate) {
                self.active = Some(candidate);
                self.last_maneuver = LastManeuver::Rotation;
                self.maybe_refresh_lock(was_grounded);
                return;
            }
        }
    }

    fn maybe_refresh_lock(&mut self, was_grounded: bool) {
        if was_grounded && self.is_grounded() && self.lock_resets < MAX_LOCK_RESETS {
            self.lock_timer = 0;
            self.lock_resets += 1;
        }
    }

    fn hard_drop(&mut self) {
        let mut rows = 0_u64;
        while self.try_translate(0, 1, false) {
            rows += 1;
        }
        self.score = self.score.saturating_add(rows.saturating_mul(2));
        self.lock_active();
    }

    fn try_hold(&mut self) {
        let Some(active) = self.active else {
            return;
        };
        if !self.hold_available {
            return;
        }
        self.hold_available = false;
        self.gravity_counter = 0;
        self.lock_timer = 0;
        self.lock_resets = 0;
        self.last_maneuver = LastManeuver::None;
        let incoming = match self.hold.replace(active.packet) {
            Some(packet) => packet,
            None => self.shift_preview(),
        };
        self.spawn(incoming);
    }

    fn is_grounded(&self) -> bool {
        self.active.is_some_and(|active| {
            self.collides(ActivePacket {
                y: active.y.saturating_add(1),
                ..active
            })
        })
    }

    fn collides(&self, active: ActivePacket) -> bool {
        active.cells().into_iter().any(|cell| {
            matrix_index(cell.x, cell.y).is_none() || self.cell(cell.x, cell.y).is_some()
        })
    }

    fn lock_active(&mut self) {
        let Some(active) = self.active.take() else {
            return;
        };
        let phase_rotation = self.is_phase_rotation(active);
        for cell in active.cells() {
            let index = matrix_index(cell.x, cell.y)
                .expect("a collision-free active packet must be inside the matrix");
            self.matrix[index] = Some(active.packet);
        }

        let rate_at_lock = self.rate;
        let cleared = self.clear_full_channels();
        self.apply_clear_score(cleared, phase_rotation, rate_at_lock);
        self.cleared_channels = self.cleared_channels.saturating_add(cleared);
        self.rate = self.cleared_channels.saturating_div(10).saturating_add(1);
        self.hold_available = true;
        self.gravity_counter = 0;
        self.lock_timer = 0;
        self.lock_resets = 0;
        self.last_maneuver = LastManeuver::None;

        if self.hidden_rows_occupied() {
            self.saturate();
        } else {
            self.pending_spawn = true;
        }
    }

    fn is_phase_rotation(&self, active: ActivePacket) -> bool {
        if !matches!(active.packet, Packet::T)
            || !matches!(self.last_maneuver, LastManeuver::Rotation)
        {
            return false;
        }
        let corners = [
            CellPoint::new(3 + active.x, 2 + active.y),
            CellPoint::new(5 + active.x, 2 + active.y),
            CellPoint::new(3 + active.x, 4 + active.y),
            CellPoint::new(5 + active.x, 4 + active.y),
        ];
        corners
            .into_iter()
            .filter(|corner| {
                matrix_index(corner.x, corner.y).is_none()
                    || self.cell(corner.x, corner.y).is_some()
            })
            .count()
            >= 3
    }

    fn clear_full_channels(&mut self) -> u32 {
        let mut destination = MATRIX_HEIGHT - 1;
        let mut cleared = 0_u32;
        for source in (0..MATRIX_HEIGHT).rev() {
            if self.row_full(source) {
                cleared += 1;
                continue;
            }
            if destination != source {
                for x in 0..MATRIX_WIDTH {
                    let source_index = matrix_index(x, source).expect("matrix coordinate");
                    let destination_index =
                        matrix_index(x, destination).expect("matrix coordinate");
                    self.matrix[destination_index] = self.matrix[source_index];
                }
            }
            destination -= 1;
        }
        for y in 0..=destination {
            for x in 0..MATRIX_WIDTH {
                let index = matrix_index(x, y).expect("matrix coordinate");
                self.matrix[index] = None;
            }
        }
        cleared
    }

    fn row_full(&self, y: i8) -> bool {
        (0..MATRIX_WIDTH).all(|x| self.cell(x, y).is_some())
    }

    fn apply_clear_score(&mut self, cleared: u32, phase: bool, rate: u32) {
        let base: u64 = match (phase, cleared) {
            (true, 0) => 400,
            (true, 1) => 800,
            (true, 2) => 1_200,
            (true, 3) => 1_600,
            (_, 1) => 100,
            (_, 2) => 300,
            (_, 3) => 500,
            (_, 4) => 800,
            _ => 0,
        };
        let qualifying = cleared == 4 || (phase && cleared > 0);
        let sustained_base = if qualifying && self.sustained_transmission {
            base.saturating_mul(3).saturating_div(2)
        } else {
            base
        };
        if cleared > 0 {
            let chain = self.signal_chain.map_or(0, |value| value.saturating_add(1));
            let chain_points = 50_u64
                .saturating_mul(u64::from(chain))
                .saturating_mul(u64::from(rate));
            self.score = self
                .score
                .saturating_add(sustained_base.saturating_mul(u64::from(rate)))
                .saturating_add(chain_points);
            self.signal_chain = Some(chain);
            self.sustained_transmission = qualifying;
            if self.matrix.iter().all(Option::is_none) {
                self.score = self
                    .score
                    .saturating_add(2_000_u64.saturating_mul(u64::from(rate)));
            }
        } else {
            self.score = self
                .score
                .saturating_add(sustained_base.saturating_mul(u64::from(rate)));
            self.signal_chain = None;
        }
    }

    fn hidden_rows_occupied(&self) -> bool {
        (0..HIDDEN_ROWS).any(|y| (0..MATRIX_WIDTH).any(|x| self.cell(x, y).is_some()))
    }

    fn spawn(&mut self, packet: Packet) {
        let active = ActivePacket {
            packet,
            rotation: Rotation::Zero,
            x: 0,
            y: 0,
        };
        if self.collides(active) {
            self.active = None;
            self.saturate();
        } else {
            self.active = Some(active);
        }
    }

    fn saturate(&mut self) {
        self.status = SignalStackStatus::Saturated;
        self.active = None;
        self.pending_spawn = false;
        self.queued_actions.clear();
    }

    fn shift_preview(&mut self) -> Packet {
        let packet = self.previews[0];
        self.previews.copy_within(1..PREVIEW_COUNT, 0);
        self.previews[PREVIEW_COUNT - 1] = self.take_from_bag();
        packet
    }

    fn take_from_bag(&mut self) -> Packet {
        if usize::from(self.bag_cursor) == self.bag.len() {
            self.bag_ordinal = self.bag_ordinal.wrapping_add(1);
            self.bag = shuffled_bag(self.seed, self.bag_ordinal);
            self.bag_cursor = 0;
        }
        let packet = self.bag[usize::from(self.bag_cursor)];
        self.bag_cursor += 1;
        packet
    }
}

fn matrix_index(x: i8, y: i8) -> Option<usize> {
    if x >= 0 && y >= 0 && x < MATRIX_WIDTH && y < MATRIX_HEIGHT {
        Some(y as usize * MATRIX_WIDTH as usize + x as usize)
    } else {
        None
    }
}

fn gravity_interval(rate: u32) -> u32 {
    const INTERVALS: [u32; 15] = [48, 43, 38, 33, 28, 23, 18, 13, 8, 6, 5, 4, 3, 2, 1];
    INTERVALS
        .get(rate.saturating_sub(1) as usize)
        .copied()
        .unwrap_or(1)
}

fn lock_delay(rate: u32) -> u8 {
    match rate {
        1..=5 => 30,
        6..=10 => 24,
        11..=14 => 18,
        _ => 12,
    }
}

fn shuffled_bag(seed: RunSeed, ordinal: u64) -> [Packet; 7] {
    let mut bytes = [0_u8; 32];
    bytes[0..8].copy_from_slice(&seed.0.to_le_bytes());
    bytes[8..16].copy_from_slice(&ordinal.to_le_bytes());
    let mut state = seed.0 ^ ordinal.rotate_left(32) ^ 0x5349_4753_5441_434B;
    for lane in 0..2 {
        state = state.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = state;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^= z >> 31;
        let start = 16 + lane * 8;
        bytes[start..start + 8].copy_from_slice(&z.to_le_bytes());
    }

    let mut rng = ChaCha8Rng::from_seed(bytes);
    let mut bag = tables::PACKET_ORDER;
    for index in (1..bag.len()).rev() {
        let bound = (index + 1) as u64;
        let range = 1_u64 << 32;
        let limit = range - range % bound;
        let value = loop {
            let candidate = u64::from(rng.next_u32());
            if candidate < limit {
                break candidate;
            }
        };
        bag.swap(index, (value % bound) as usize);
    }
    bag
}

struct Fnv1a(u64);

impl Fnv1a {
    const fn new() -> Self {
        Self(14_695_981_039_346_656_037)
    }

    const fn finish(self) -> u64 {
        self.0
    }

    fn bytes(&mut self, bytes: &[u8]) {
        for byte in bytes {
            self.0 ^= u64::from(*byte);
            self.0 = self.0.wrapping_mul(1_099_511_628_211);
        }
    }

    fn u8(&mut self, value: u8) {
        self.bytes(&[value]);
    }

    fn i8(&mut self, value: i8) {
        self.bytes(&value.to_le_bytes());
    }

    fn u16(&mut self, value: u16) {
        self.bytes(&value.to_le_bytes());
    }

    fn u32(&mut self, value: u32) {
        self.bytes(&value.to_le_bytes());
    }

    fn u64(&mut self, value: u64) {
        self.bytes(&value.to_le_bytes());
    }

    fn bool(&mut self, value: bool) {
        self.u8(u8::from(value));
    }

    fn option<T>(&mut self, value: Option<T>, encode: impl FnOnce(&mut Self, T)) {
        match value {
            Some(value) => {
                self.u8(1);
                encode(self, value);
            }
            None => self.u8(0),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use super::*;

    fn step(game: &mut SignalStack, tick: u64, actions: &[GameAction]) {
        for action in actions {
            game.handle_action(*action);
        }
        game.update(SimulationStep {
            tick: SimulationTick(tick),
        });
    }

    fn set_cell(game: &mut SignalStack, x: i8, y: i8, packet: Option<Packet>) {
        game.matrix[matrix_index(x, y).expect("test coordinate")] = packet;
    }

    #[test]
    fn spawn_tables_match_revision_one() {
        let expected = [
            (Packet::I, [(3, 2), (4, 2), (5, 2), (6, 2)]),
            (Packet::J, [(3, 2), (3, 3), (4, 3), (5, 3)]),
            (Packet::L, [(5, 2), (3, 3), (4, 3), (5, 3)]),
            (Packet::O, [(4, 2), (5, 2), (4, 3), (5, 3)]),
            (Packet::S, [(4, 2), (5, 2), (3, 3), (4, 3)]),
            (Packet::T, [(4, 2), (3, 3), (4, 3), (5, 3)]),
            (Packet::Z, [(3, 2), (4, 2), (4, 3), (5, 3)]),
        ];
        for (packet, coordinates) in expected {
            let actual = *tables::cells(packet, Rotation::Zero);
            assert_eq!(
                actual,
                coordinates.map(|(x, y)| CellPoint::new(x, y)),
                "{packet:?}"
            );
        }
    }

    #[test]
    fn every_packet_rotates_legally_at_walls_and_floor() {
        for packet in tables::PACKET_ORDER {
            for rotation in [
                Rotation::Zero,
                Rotation::Right,
                Rotation::Two,
                Rotation::Left,
            ] {
                for edge in [-1, 1] {
                    let mut game = SignalStack::new(RunSeed(1));
                    game.active = Some(ActivePacket {
                        packet,
                        rotation,
                        x: 0,
                        y: 0,
                    });
                    while game.try_translate(edge, 0, false) {}
                    while game.try_translate(0, 1, false) {}
                    let before = game.active.expect("active");
                    game.try_rotate(edge > 0);
                    let after = game.active.expect("active");
                    assert!(!game.collides(after), "{packet:?} {rotation:?} {edge}");
                    if !matches!(packet, Packet::O) {
                        assert_ne!(
                            before.rotation, after.rotation,
                            "{packet:?} {rotation:?} {edge}"
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn lock_delays_match_each_rate_band() {
        assert_eq!(lock_delay(1), 30);
        assert_eq!(lock_delay(5), 30);
        assert_eq!(lock_delay(6), 24);
        assert_eq!(lock_delay(10), 24);
        assert_eq!(lock_delay(11), 18);
        assert_eq!(lock_delay(14), 18);
        assert_eq!(lock_delay(15), 12);
        assert_eq!(lock_delay(u32::MAX), 12);

        let mut game = SignalStack::new(RunSeed(1));
        game.active = Some(ActivePacket {
            packet: Packet::O,
            rotation: Rotation::Zero,
            x: 0,
            y: 20,
        });
        for tick in 1..30 {
            step(&mut game, tick, &[]);
            assert!(game.active().is_some());
        }
        step(&mut game, 30, &[]);
        assert!(game.active().is_none());
        assert!(game.pending_spawn());
    }

    #[test]
    fn o_rotation_is_an_accepted_lock_neutral_noop() {
        let mut game = SignalStack::new(RunSeed(1));
        game.active = Some(ActivePacket {
            packet: Packet::O,
            rotation: Rotation::Zero,
            x: 0,
            y: 20,
        });
        game.lock_timer = 11;
        game.lock_resets = 3;
        game.last_maneuver = LastManeuver::Lateral;
        game.try_rotate(true);
        assert_eq!(
            game.active(),
            Some(ActivePacket {
                packet: Packet::O,
                rotation: Rotation::Zero,
                x: 0,
                y: 20,
            })
        );
        assert_eq!(game.lock_timer(), 11);
        assert_eq!(game.lock_resets(), 3);
        assert_eq!(game.last_maneuver(), LastManeuver::Lateral);
    }

    #[test]
    fn bags_are_seeded_independent_permutations() {
        let first = shuffled_bag(RunSeed(0x1234_5678), 0);
        assert_eq!(
            first,
            [
                Packet::O,
                Packet::S,
                Packet::J,
                Packet::I,
                Packet::Z,
                Packet::L,
                Packet::T,
            ]
        );
        assert_eq!(first, shuffled_bag(RunSeed(0x1234_5678), 0));
        assert_ne!(first, shuffled_bag(RunSeed(0x1234_5678), 1));
        assert_eq!(
            first.into_iter().collect::<BTreeSet<_>>(),
            tables::PACKET_ORDER.into_iter().collect()
        );
    }

    #[test]
    fn preview_contains_five_and_hold_is_once_per_lock() {
        let mut game = SignalStack::new(RunSeed(9));
        let first = game.active().expect("spawn").packet;
        let next = game.previews()[0];
        game.apply_action(GameAction::Hold);
        assert_eq!(game.hold(), Some(first));
        assert_eq!(game.active().expect("active").packet, next);
        assert!(!game.hold_available());
        let held_state = game.state_hash();
        game.apply_action(GameAction::Hold);
        assert_eq!(game.state_hash(), held_state);
        game.hard_drop();
        assert!(game.hold_available());
    }

    #[test]
    fn soft_and_hard_drop_award_exact_points() {
        let mut game = SignalStack::new(RunSeed(4));
        let start_y = game.active().expect("active").y;
        game.apply_action(GameAction::SoftDrop);
        assert_eq!(game.active().expect("active").y, start_y + 1);
        assert_eq!(game.score(), 1);
        let before_hard_drop = game.active().expect("active");
        let expected_rows = (0..MATRIX_HEIGHT)
            .take_while(|distance| {
                !game.collides(ActivePacket {
                    y: before_hard_drop.y + *distance + 1,
                    ..before_hard_drop
                })
            })
            .count() as u64;
        game.apply_action(GameAction::HardDrop);
        assert_eq!(game.score(), 1 + expected_rows * 2);
        assert!(game.pending_spawn());
    }

    #[test]
    fn gravity_and_rate_intervals_are_exact() {
        assert_eq!(
            (1..=15).map(gravity_interval).collect::<Vec<_>>(),
            vec![48, 43, 38, 33, 28, 23, 18, 13, 8, 6, 5, 4, 3, 2, 1]
        );
        assert_eq!(gravity_interval(99), 1);
        let mut game = SignalStack::new(RunSeed(1));
        let start_y = game.active().expect("active").y;
        for tick in 1..48 {
            step(&mut game, tick, &[]);
        }
        assert_eq!(game.active().expect("active").y, start_y);
        step(&mut game, 48, &[]);
        assert_eq!(game.active().expect("active").y, start_y + 1);
    }

    #[test]
    fn grounded_maneuvers_refresh_lock_at_most_fifteen_times() {
        let mut game = SignalStack::new(RunSeed(3));
        let active = game.active.expect("active");
        game.active = Some(ActivePacket { y: 20, ..active });
        while !game.is_grounded() {
            game.try_translate(0, 1, false);
        }
        game.lock_timer = 10;
        for index in 0..20 {
            game.try_translate(if index % 2 == 0 { -1 } else { 1 }, 0, true);
            game.lock_timer = 10;
        }
        assert_eq!(game.lock_resets(), 15);
        game.try_translate(-1, 0, true);
        assert_eq!(game.lock_timer(), 10);
    }

    #[test]
    fn clear_scoring_chain_sustain_and_zero_state_are_exact() {
        let mut game = SignalStack::new(RunSeed(0));
        game.matrix.fill(None);
        game.apply_clear_score(4, false, 2);
        assert_eq!(game.score(), 1_600 + 4_000);
        game.matrix[0] = Some(Packet::I);
        game.apply_clear_score(4, false, 2);
        assert_eq!(game.score(), 5_600 + 2_400 + 100);
        game.apply_clear_score(1, false, 2);
        assert_eq!(game.score(), 8_100 + 200 + 200);
        game.apply_clear_score(0, false, 2);
        assert_eq!(game.signal_chain, None);
        assert!(!game.sustained_transmission);
    }

    #[test]
    fn channels_clear_and_rate_advances_after_scoring() {
        let mut game = SignalStack::new(RunSeed(12));
        game.matrix.fill(None);
        for y in 14..24 {
            for x in 0..MATRIX_WIDTH {
                set_cell(&mut game, x, y, Some(Packet::J));
            }
        }
        let cleared = game.clear_full_channels();
        assert_eq!(cleared, 10);
        game.apply_clear_score(cleared, false, game.rate);
        game.cleared_channels += cleared;
        game.rate = game.cleared_channels / 10 + 1;
        assert_eq!(game.rate(), 2);
    }

    #[test]
    fn score_arithmetic_saturates() {
        let mut game = SignalStack::new(RunSeed(1));
        game.score = u64::MAX - 1;
        game.apply_clear_score(4, false, u32::MAX);
        assert_eq!(game.score(), u64::MAX);
    }

    #[test]
    fn hidden_occupancy_and_spawn_failure_saturate_run() {
        let mut hidden = SignalStack::new(RunSeed(2));
        hidden.matrix.fill(None);
        set_cell(&mut hidden, 0, 0, Some(Packet::Z));
        hidden.active = Some(ActivePacket {
            packet: Packet::O,
            rotation: Rotation::Zero,
            x: 0,
            y: 20,
        });
        hidden.lock_active();
        assert_eq!(hidden.status(), SignalStackStatus::Saturated);

        let mut spawn = SignalStack::new(RunSeed(2));
        spawn.active = None;
        for point in tables::cells(Packet::T, Rotation::Zero) {
            set_cell(&mut spawn, point.x, point.y, Some(Packet::I));
        }
        spawn.spawn(Packet::T);
        assert_eq!(spawn.status(), SignalStackStatus::Saturated);
        assert!(spawn.result().is_some());
    }

    #[test]
    fn locked_packet_spawns_successor_on_next_tick() {
        let mut game = SignalStack::new(RunSeed(21));
        let next = game.previews()[0];
        game.hard_drop();
        assert!(game.active().is_none());
        assert!(game.pending_spawn());
        step(&mut game, 1, &[]);
        assert_eq!(game.active().expect("next packet").packet, next);
    }

    #[test]
    fn pause_freezes_authoritative_state_except_status() {
        let mut game = SignalStack::new(RunSeed(7));
        game.set_paused(true);
        let active = game.active();
        let tick = game.tick;
        step(&mut game, 99, &[GameAction::HardDrop]);
        assert_eq!(game.active(), active);
        assert_eq!(game.score(), 0);
        assert_eq!(game.tick, tick);
        game.set_paused(false);
        assert_eq!(game.status(), SignalStackStatus::Running);
    }

    #[test]
    fn lock_resolves_four_channels_and_zero_state_bonus() {
        let mut game = SignalStack::new(RunSeed(8));
        game.matrix.fill(None);
        for y in 20..24 {
            for x in 0..MATRIX_WIDTH {
                if x != 5 {
                    set_cell(&mut game, x, y, Some(Packet::J));
                }
            }
        }
        game.active = Some(ActivePacket {
            packet: Packet::I,
            rotation: Rotation::Right,
            x: 0,
            y: 19,
        });
        game.last_maneuver = LastManeuver::None;
        game.lock_active();
        assert_eq!(game.cleared_channels(), 4);
        assert_eq!(game.score(), 2_800);
        assert!(game.matrix().iter().all(Option::is_none));
    }

    #[test]
    fn t_phase_rotation_scores_with_three_blocked_corners() {
        let mut game = SignalStack::new(RunSeed(10));
        game.matrix.fill(None);
        game.active = Some(ActivePacket {
            packet: Packet::T,
            rotation: Rotation::Zero,
            x: 0,
            y: 19,
        });
        for (x, y) in [(3, 21), (5, 21), (3, 23)] {
            set_cell(&mut game, x, y, Some(Packet::Z));
        }
        game.last_maneuver = LastManeuver::Rotation;
        game.lock_active();
        assert_eq!(game.score(), 400);
        assert_eq!(game.cleared_channels(), 0);
    }

    #[test]
    fn deterministic_actions_produce_golden_hash() {
        let mut first = SignalStack::new(RunSeed(0xCAFE_BABE));
        let mut second = first.clone();
        for tick in 1..=180 {
            let actions: &[GameAction] = match tick {
                3 | 5 | 7 => &[GameAction::MoveLeft],
                11 => &[GameAction::RotateClockwise],
                15 => &[GameAction::Hold],
                24 | 48 | 72 => &[GameAction::SoftDrop],
                90 => &[GameAction::HardDrop],
                100 => &[GameAction::MoveRight, GameAction::RotateCounterclockwise],
                150 => &[GameAction::HardDrop],
                _ => &[],
            };
            step(&mut first, tick, actions);
            step(&mut second, tick, actions);
        }
        assert_eq!(first.state_hash(), second.state_hash());
        assert_eq!(first.state_hash(), StateHash(17_381_950_295_200_256_755));
    }

    #[test]
    fn duration_ticks_are_relative_to_the_run() {
        let mut game = SignalStack::new(RunSeed(1));
        game.update(SimulationStep {
            tick: SimulationTick(10_000),
        });
        game.update(SimulationStep {
            tick: SimulationTick(20_000),
        });

        assert_eq!(game.tick(), SimulationTick(2));
    }
}

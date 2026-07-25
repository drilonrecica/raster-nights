// SPDX-License-Identifier: MPL-2.0

//! Deterministic revision-1 rules for Loopback Quick Circuit.

mod render;

pub use render::render;

use rand_chacha::ChaCha8Rng;
use rand_core::{Rng, SeedableRng};
use raster_engine::{
    GameAction, GameId, GameOutcome, GameResult, GameStatus, ModeId, RulesRevision, RunSeed,
    SimulationStep, SimulationTick, StateHash,
};

pub const ARENA_WIDTH: i8 = 24;
pub const ARENA_HEIGHT: i8 = 20;
pub const RUN_DURATION_TICKS: u64 = 7_200;
pub const RECOVERY_TICKS: u8 = 60;
pub const STARTING_INTEGRITY: u8 = 3;
pub const MAX_MULTIPLIER: u8 = 4;

const START_ROUTE: [Point; 4] = [
    Point::new(13, 10),
    Point::new(12, 10),
    Point::new(11, 10),
    Point::new(10, 10),
];

/// Fixed paired ports for Quick Circuit revision 1.
pub const PORTS: [PortPair; 2] = [
    PortPair::new(Point::new(3, 3), Point::new(20, 16)),
    PortPair::new(Point::new(20, 3), Point::new(3, 16)),
];

#[must_use]
pub fn rules_revision() -> RulesRevision {
    RulesRevision::new(1).expect("revision one is nonzero")
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct Point {
    pub x: i8,
    pub y: i8,
}

impl Point {
    pub const fn new(x: i8, y: i8) -> Self {
        Self { x, y }
    }

    const fn translated(self, direction: Direction) -> Self {
        let (dx, dy) = direction.delta();
        Self::new(self.x.saturating_add(dx), self.y.saturating_add(dy))
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct PortPair {
    pub first: Point,
    pub second: Point,
}

impl PortPair {
    pub const fn new(first: Point, second: Point) -> Self {
        Self { first, second }
    }

    const fn destination(self, point: Point) -> Option<Point> {
        if point.x == self.first.x && point.y == self.first.y {
            Some(self.second)
        } else if point.x == self.second.x && point.y == self.second.y {
            Some(self.first)
        } else {
            None
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[repr(u8)]
pub enum Direction {
    Left = 1,
    Right = 2,
    Up = 3,
    Down = 4,
}

impl Direction {
    const fn delta(self) -> (i8, i8) {
        match self {
            Self::Left => (-1, 0),
            Self::Right => (1, 0),
            Self::Up => (0, -1),
            Self::Down => (0, 1),
        }
    }

    const fn opposite(self) -> Self {
        match self {
            Self::Left => Self::Right,
            Self::Right => Self::Left,
            Self::Up => Self::Down,
            Self::Down => Self::Up,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum LoopbackStatus {
    Running = 1,
    Paused = 2,
    Completed = 3,
    Disconnected = 4,
}

/// Short, host-independent description suitable for an accessibility status region.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SemanticStatus {
    pub summary: String,
    pub detail: String,
}

/// Authoritative Quick Circuit simulation.
#[derive(Clone, Debug)]
pub struct Loopback {
    seed: RunSeed,
    status: LoopbackStatus,
    tick: SimulationTick,
    route: Vec<Point>,
    direction: Direction,
    pending_direction: Option<Direction>,
    payload: Point,
    payload_ordinal: u64,
    score: u64,
    payloads_collected: u32,
    multiplier: u8,
    integrity: u8,
    recovery_ticks: u8,
    movement_ticks: u8,
}

impl Loopback {
    #[must_use]
    pub fn new(seed: RunSeed) -> Self {
        let mut game = Self {
            seed,
            status: LoopbackStatus::Running,
            tick: SimulationTick(0),
            route: START_ROUTE.to_vec(),
            direction: Direction::Right,
            pending_direction: None,
            payload: Point::new(0, 0),
            payload_ordinal: 0,
            score: 0,
            payloads_collected: 0,
            multiplier: 1,
            integrity: STARTING_INTEGRITY,
            recovery_ticks: 0,
            movement_ticks: 0,
        };
        game.spawn_payload();
        game
    }

    pub fn reset(&mut self, seed: RunSeed) {
        *self = Self::new(seed);
    }

    /// Queues at most one heading change before the next route movement.
    pub fn handle_action(&mut self, action: GameAction) {
        if !matches!(self.status, LoopbackStatus::Running) || self.pending_direction.is_some() {
            return;
        }
        let requested = match action {
            GameAction::MoveLeft => Some(Direction::Left),
            GameAction::MoveRight => Some(Direction::Right),
            GameAction::MoveUp => Some(Direction::Up),
            GameAction::MoveDown => Some(Direction::Down),
            GameAction::RotateClockwise
            | GameAction::RotateCounterclockwise
            | GameAction::SoftDrop
            | GameAction::HardDrop
            | GameAction::Hold
            | GameAction::Primary
            | GameAction::Secondary => None,
        };
        if requested.is_some_and(|direction| direction != self.direction.opposite()) {
            self.pending_direction = requested;
        }
    }

    /// Advances one fixed 60 Hz simulation tick.
    pub fn update(&mut self, _step: SimulationStep) {
        if !matches!(self.status, LoopbackStatus::Running) {
            self.pending_direction = None;
            return;
        }

        self.tick = SimulationTick(self.tick.0.saturating_add(1));
        self.recovery_ticks = self.recovery_ticks.saturating_sub(1);

        if self.tick.0 >= RUN_DURATION_TICKS {
            self.score = self
                .score
                .saturating_add(500_u64.saturating_mul(u64::from(self.integrity)));
            self.status = LoopbackStatus::Completed;
            self.pending_direction = None;
            return;
        }

        self.movement_ticks = self.movement_ticks.saturating_add(1);
        if self.movement_ticks >= movement_interval(self.payloads_collected) {
            self.movement_ticks = 0;
            self.move_route();
        }
    }

    pub fn set_paused(&mut self, paused: bool) {
        match (self.status, paused) {
            (LoopbackStatus::Running, true) => {
                self.status = LoopbackStatus::Paused;
                self.pending_direction = None;
            }
            (LoopbackStatus::Paused, false) => self.status = LoopbackStatus::Running,
            _ => {}
        }
    }

    #[must_use]
    pub const fn status(&self) -> LoopbackStatus {
        self.status
    }

    #[must_use]
    pub const fn game_status(&self) -> GameStatus {
        match self.status {
            LoopbackStatus::Running | LoopbackStatus::Paused => GameStatus::Running,
            LoopbackStatus::Completed | LoopbackStatus::Disconnected => GameStatus::Finished,
        }
    }

    #[must_use]
    pub fn result(&self) -> Option<GameResult> {
        let outcome = match self.status {
            LoopbackStatus::Completed => GameOutcome::Completed,
            LoopbackStatus::Disconnected => GameOutcome::GameOver,
            LoopbackStatus::Running | LoopbackStatus::Paused => return None,
        };
        Some(GameResult {
            game_id: GameId::parse("loopback").expect("static game identifier is valid"),
            mode_id: ModeId::parse("quick-circuit").expect("static mode identifier is valid"),
            rules_revision: rules_revision(),
            seed: self.seed,
            final_tick: self.tick,
            score: self.score,
            outcome,
            final_state_hash: self.state_hash(),
            discoveries: Vec::new(),
        })
    }

    #[must_use]
    pub const fn tick(&self) -> SimulationTick {
        self.tick
    }

    #[must_use]
    pub fn route(&self) -> &[Point] {
        &self.route
    }

    #[must_use]
    pub const fn direction(&self) -> Direction {
        self.direction
    }

    #[must_use]
    pub const fn payload(&self) -> Point {
        self.payload
    }

    #[must_use]
    pub const fn score(&self) -> u64 {
        self.score
    }

    #[must_use]
    pub const fn payloads_collected(&self) -> u32 {
        self.payloads_collected
    }

    #[must_use]
    pub const fn multiplier(&self) -> u8 {
        self.multiplier
    }

    #[must_use]
    pub const fn integrity(&self) -> u8 {
        self.integrity
    }

    #[must_use]
    pub const fn recovery_ticks(&self) -> u8 {
        self.recovery_ticks
    }

    #[must_use]
    pub const fn remaining_ticks(&self) -> u64 {
        RUN_DURATION_TICKS.saturating_sub(self.tick.0)
    }

    #[must_use]
    pub fn semantic_status(&self) -> SemanticStatus {
        let state = match self.status {
            LoopbackStatus::Running if self.recovery_ticks > 0 => "recovering",
            LoopbackStatus::Running => "running",
            LoopbackStatus::Paused => "paused",
            LoopbackStatus::Completed => "circuit completed",
            LoopbackStatus::Disconnected => "route disconnected",
        };
        SemanticStatus {
            summary: format!(
                "Loopback {state}. Score {}. Integrity {} of {}.",
                self.score, self.integrity, STARTING_INTEGRITY
            ),
            detail: format!(
                "{} payloads collected. Next payload multiplier {}. {} seconds remaining.",
                self.payloads_collected,
                self.multiplier,
                self.remaining_ticks().div_ceil(60)
            ),
        }
    }

    #[must_use]
    pub fn state_hash(&self) -> StateHash {
        let mut hash = Fnv1a::new();
        hash.u16(rules_revision().get());
        hash.u64(self.seed.0);
        hash.u8(self.status as u8);
        hash.u64(self.tick.0);
        hash.u32(self.route.len() as u32);
        for point in &self.route {
            hash.point(*point);
        }
        hash.u8(self.direction as u8);
        hash.u8(self
            .pending_direction
            .map_or(0, |direction| direction as u8));
        hash.point(self.payload);
        hash.u64(self.payload_ordinal);
        hash.u64(self.score);
        hash.u32(self.payloads_collected);
        hash.u8(self.multiplier);
        hash.u8(self.integrity);
        hash.u8(self.recovery_ticks);
        hash.u8(self.movement_ticks);
        StateHash(hash.finish())
    }

    fn move_route(&mut self) {
        if let Some(direction) = self.pending_direction.take() {
            self.direction = direction;
        }
        let entered = self.route[0].translated(self.direction);
        let (next, traversed_port) =
            port_destination(entered).map_or((entered, false), |destination| (destination, true));
        let collecting = next == self.payload;
        // Moving into the current tail is legal when the tail moves away.
        let occupied_length = self.route.len().saturating_sub(usize::from(!collecting));
        let collision = !inside_arena(next) || self.route[..occupied_length].contains(&next);
        if collision {
            if self.recovery_ticks == 0 {
                self.damage();
            }
            return;
        }

        self.route.insert(0, next);
        if collecting {
            self.score = self
                .score
                .saturating_add(100_u64.saturating_mul(u64::from(self.multiplier)));
            self.payloads_collected = self.payloads_collected.saturating_add(1);
            self.multiplier = 1;
            self.spawn_payload();
        } else {
            self.route.pop();
        }
        if traversed_port {
            self.multiplier = self.multiplier.saturating_add(1).min(MAX_MULTIPLIER);
        }
    }

    fn damage(&mut self) {
        self.integrity = self.integrity.saturating_sub(1);
        self.multiplier = 1;
        self.pending_direction = None;
        if self.integrity == 0 {
            self.status = LoopbackStatus::Disconnected;
            self.recovery_ticks = 0;
            return;
        }
        self.route.clear();
        self.route.extend_from_slice(&START_ROUTE);
        self.direction = Direction::Right;
        self.recovery_ticks = RECOVERY_TICKS;
        self.movement_ticks = 0;
        if self.route.contains(&self.payload) {
            self.spawn_payload();
        }
    }

    fn spawn_payload(&mut self) {
        let ordinal = self.payload_ordinal;
        self.payload_ordinal = self.payload_ordinal.saturating_add(1);
        let mixed_seed = self.seed.0 ^ ordinal.wrapping_add(0x9E37_79B9_7F4A_7C15).rotate_left(27);
        let mut rng = ChaCha8Rng::seed_from_u64(mixed_seed);
        let cell_count = u16::from(ARENA_WIDTH as u8) * u16::from(ARENA_HEIGHT as u8);
        let start = rng.next_u32() as u16 % cell_count;
        let stride = coprime_stride(rng.next_u32() as u16 % cell_count);
        for offset in 0..cell_count {
            let index = ((u32::from(start) + u32::from(offset).saturating_mul(u32::from(stride)))
                % u32::from(cell_count)) as u16;
            let point = Point::new(
                (index % u16::from(ARENA_WIDTH as u8)) as i8,
                (index / u16::from(ARENA_WIDTH as u8)) as i8,
            );
            if !self.route.contains(&point) && port_destination(point).is_none() {
                self.payload = point;
                return;
            }
        }
        // A legal run cannot fill all 476 non-port cells.
        debug_assert!(false, "Quick Circuit arena has no free payload cell");
    }
}

#[must_use]
pub const fn movement_interval(payloads: u32) -> u8 {
    match payloads {
        0..=7 => 12,
        8..=15 => 10,
        16..=23 => 8,
        24..=31 => 7,
        _ => 6,
    }
}

#[must_use]
pub fn port_destination(point: Point) -> Option<Point> {
    PORTS.into_iter().find_map(|pair| pair.destination(point))
}

const fn inside_arena(point: Point) -> bool {
    point.x >= 0 && point.x < ARENA_WIDTH && point.y >= 0 && point.y < ARENA_HEIGHT
}

fn coprime_stride(candidate: u16) -> u16 {
    let mut stride = candidate.max(1);
    while gcd(
        stride,
        u16::from(ARENA_WIDTH as u8) * u16::from(ARENA_HEIGHT as u8),
    ) != 1
    {
        stride = stride.saturating_add(1);
    }
    stride
}

const fn gcd(mut left: u16, mut right: u16) -> u16 {
    while right != 0 {
        let remainder = left % right;
        left = right;
        right = remainder;
    }
    left
}

/// Shared input sequence used by native and Wasm determinism tests.
#[must_use]
pub fn golden_run() -> StateHash {
    let mut game = Loopback::new(RunSeed(0x10_0B_AC));
    for tick in 1..=900 {
        let action = match tick {
            36 => Some(GameAction::MoveDown),
            156 => Some(GameAction::MoveLeft),
            276 => Some(GameAction::MoveUp),
            396 => Some(GameAction::MoveRight),
            516 => Some(GameAction::MoveDown),
            636 => Some(GameAction::MoveRight),
            756 => Some(GameAction::MoveUp),
            _ => None,
        };
        if let Some(action) = action {
            game.handle_action(action);
        }
        game.update(SimulationStep {
            tick: SimulationTick(tick),
        });
    }
    game.state_hash()
}

struct Fnv1a(u64);

impl Fnv1a {
    const fn new() -> Self {
        Self(14_695_981_039_346_656_037)
    }

    fn bytes(&mut self, bytes: &[u8]) {
        for byte in bytes {
            self.0 = (self.0 ^ u64::from(*byte)).wrapping_mul(1_099_511_628_211);
        }
    }

    fn u8(&mut self, value: u8) {
        self.bytes(&[value]);
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

    fn point(&mut self, point: Point) {
        self.u8(point.x as u8);
        self.u8(point.y as u8);
    }

    const fn finish(self) -> u64 {
        self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn advance(game: &mut Loopback, ticks: u64) {
        for tick in 1..=ticks {
            game.update(SimulationStep {
                tick: SimulationTick(tick),
            });
        }
    }

    fn move_once(game: &mut Loopback) {
        advance(game, u64::from(movement_interval(game.payloads_collected)));
    }

    #[test]
    fn starts_with_documented_route_timer_and_integrity() {
        let game = Loopback::new(RunSeed(1));

        assert_eq!(game.route(), START_ROUTE);
        assert_eq!(game.direction(), Direction::Right);
        assert_eq!(game.integrity(), 3);
        assert_eq!(game.remaining_ticks(), 7_200);
        assert!(!game.route().contains(&game.payload()));
        assert!(port_destination(game.payload()).is_none());
    }

    #[test]
    fn direct_reversal_is_rejected_and_only_one_turn_is_queued() {
        let mut game = Loopback::new(RunSeed(1));
        game.handle_action(GameAction::MoveLeft);
        game.handle_action(GameAction::MoveDown);
        move_once(&mut game);
        assert_eq!(game.direction(), Direction::Down);

        game.handle_action(GameAction::MoveRight);
        game.handle_action(GameAction::MoveUp);
        move_once(&mut game);
        assert_eq!(game.direction(), Direction::Right);
    }

    #[test]
    fn payload_spawning_is_seeded_and_avoids_route_and_ports() {
        let first = Loopback::new(RunSeed(42));
        let second = Loopback::new(RunSeed(42));
        let different = Loopback::new(RunSeed(43));

        assert_eq!(first.payload(), second.payload());
        assert_ne!(first.payload(), different.payload());
        assert!(!first.route().contains(&first.payload()));
        assert!(port_destination(first.payload()).is_none());
    }

    #[test]
    fn ports_preserve_heading_and_raise_multiplier_to_cap() {
        let mut game = Loopback::new(RunSeed(1));
        game.route = vec![
            Point::new(2, 3),
            Point::new(1, 3),
            Point::new(0, 3),
            Point::new(0, 2),
        ];
        game.direction = Direction::Right;
        game.multiplier = MAX_MULTIPLIER;
        move_once(&mut game);

        assert_eq!(game.route()[0], Point::new(20, 16));
        assert_eq!(game.direction(), Direction::Right);
        assert_eq!(game.multiplier(), MAX_MULTIPLIER);
    }

    #[test]
    fn collecting_scores_current_multiplier_grows_route_and_resets_multiplier() {
        let mut game = Loopback::new(RunSeed(1));
        game.payload = Point::new(14, 10);
        game.multiplier = 3;
        move_once(&mut game);

        assert_eq!(game.score(), 300);
        assert_eq!(game.payloads_collected(), 1);
        assert_eq!(game.route().len(), 5);
        assert_eq!(game.multiplier(), 1);
    }

    #[test]
    fn speed_thresholds_are_exact() {
        assert_eq!(
            [0, 7, 8, 15, 16, 23, 24, 31, 32].map(movement_interval),
            [12, 12, 10, 10, 8, 8, 7, 7, 6]
        );
    }

    #[test]
    fn collision_damages_resets_and_recovery_prevents_repeat_damage() {
        let mut game = Loopback::new(RunSeed(1));
        game.route = vec![
            Point::new(23, 8),
            Point::new(22, 8),
            Point::new(21, 8),
            Point::new(20, 8),
        ];
        game.direction = Direction::Right;
        game.multiplier = 4;
        move_once(&mut game);

        assert_eq!(game.integrity(), 2);
        assert_eq!(game.route(), START_ROUTE);
        assert_eq!(game.multiplier(), 1);
        assert_eq!(game.recovery_ticks(), RECOVERY_TICKS);

        game.route = vec![
            Point::new(23, 8),
            Point::new(22, 8),
            Point::new(21, 8),
            Point::new(20, 8),
        ];
        game.direction = Direction::Right;
        move_once(&mut game);
        assert_eq!(game.integrity(), 2);
        assert!(game.recovery_ticks() < RECOVERY_TICKS);
    }

    #[test]
    fn self_collision_uses_same_damage_path() {
        let mut game = Loopback::new(RunSeed(1));
        game.route = vec![
            Point::new(5, 5),
            Point::new(5, 6),
            Point::new(4, 6),
            Point::new(4, 5),
            Point::new(4, 4),
            Point::new(5, 4),
        ];
        game.direction = Direction::Left;
        move_once(&mut game);
        assert_eq!(game.integrity(), 2);
        assert_eq!(game.route(), START_ROUTE);
    }

    #[test]
    fn zero_integrity_is_game_over() {
        let mut game = Loopback::new(RunSeed(1));
        game.integrity = 1;
        game.route[0] = Point::new(23, 10);
        game.direction = Direction::Right;
        move_once(&mut game);

        assert_eq!(game.status(), LoopbackStatus::Disconnected);
        assert_eq!(game.game_status(), GameStatus::Finished);
        assert_eq!(
            game.result().expect("finished").outcome,
            GameOutcome::GameOver
        );
    }

    #[test]
    fn timer_completion_awards_integrity_bonus_once() {
        let mut game = Loopback::new(RunSeed(1));
        game.tick = SimulationTick(RUN_DURATION_TICKS - 1);
        game.score = 250;
        game.integrity = 2;
        advance(&mut game, 1);

        assert_eq!(game.status(), LoopbackStatus::Completed);
        assert_eq!(game.score(), 1_250);
        assert_eq!(
            game.result().expect("finished").outcome,
            GameOutcome::Completed
        );
        advance(&mut game, 10);
        assert_eq!(game.score(), 1_250);
    }

    #[test]
    fn pause_freezes_all_authoritative_timers_and_actions() {
        let mut game = Loopback::new(RunSeed(1));
        game.set_paused(true);
        game.handle_action(GameAction::MoveDown);
        advance(&mut game, 100);

        assert_eq!(game.tick(), SimulationTick(0));
        assert_eq!(game.route(), START_ROUTE);
        game.set_paused(false);
        assert_eq!(game.status(), LoopbackStatus::Running);
    }

    #[test]
    fn golden_run_is_repeatable() {
        assert_eq!(golden_run(), golden_run());
        assert_eq!(golden_run(), StateHash(9_267_901_411_767_299_672));
    }
}

#[cfg(all(test, target_arch = "wasm32"))]
mod wasm_tests {
    use super::*;
    use wasm_bindgen_test::wasm_bindgen_test;

    #[wasm_bindgen_test]
    fn quick_circuit_golden_run_matches_native() {
        assert_eq!(golden_run(), StateHash(9_267_901_411_767_299_672));
    }
}

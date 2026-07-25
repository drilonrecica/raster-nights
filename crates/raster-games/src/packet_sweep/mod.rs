// SPDX-License-Identifier: MPL-2.0

//! Deterministic revision-1 rules for the hidden Packet Sweep maintenance game.

mod render;

pub use render::render;

use raster_engine::{
    GameAction, GameId, GameOutcome, GameResult, GameStatus, ModeId, RulesRevision, RunSeed,
    SimulationStep, SimulationTick, StateHash,
};

pub const ARENA_WIDTH: i16 = 24;
pub const ARENA_HEIGHT: i16 = 18;
pub const RUN_TICKS: u64 = 5_400;
pub const RECOVERY_TICKS: u8 = 60;
pub const MAX_ERRORS: usize = 8;

const ERROR_MOVE_INTERVAL: u64 = 6;
const START: Point = Point::new(12, 9);
const FIXED_WALLS: [Point; 16] = [
    Point::new(5, 4),
    Point::new(5, 5),
    Point::new(5, 6),
    Point::new(5, 11),
    Point::new(5, 12),
    Point::new(5, 13),
    Point::new(18, 4),
    Point::new(18, 5),
    Point::new(18, 6),
    Point::new(18, 11),
    Point::new(18, 12),
    Point::new(18, 13),
    Point::new(10, 3),
    Point::new(13, 3),
    Point::new(10, 14),
    Point::new(13, 14),
];

#[must_use]
pub fn rules_revision() -> RulesRevision {
    RulesRevision::new(1).expect("revision one is nonzero")
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct Point {
    pub x: i16,
    pub y: i16,
}

impl Point {
    #[must_use]
    pub const fn new(x: i16, y: i16) -> Self {
        Self { x, y }
    }

    const fn moved(self, heading: Heading) -> Self {
        let (dx, dy) = heading.delta();
        Self::new(self.x + dx, self.y + dy)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum Heading {
    Up = 1,
    Right = 2,
    Down = 3,
    Left = 4,
}

impl Heading {
    const fn delta(self) -> (i16, i16) {
        match self {
            Self::Up => (0, -1),
            Self::Right => (1, 0),
            Self::Down => (0, 1),
            Self::Left => (-1, 0),
        }
    }

    const fn reflected(self) -> Self {
        match self {
            Self::Up => Self::Down,
            Self::Right => Self::Left,
            Self::Down => Self::Up,
            Self::Left => Self::Right,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ChecksumError {
    pub position: Point,
    pub heading: Heading,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum PacketSweepStatus {
    Running = 1,
    Paused = 2,
    Completed = 3,
    Failed = 4,
}

#[derive(Clone, Debug)]
pub struct PacketSweep {
    seed: RunSeed,
    tick: SimulationTick,
    status: PacketSweepStatus,
    cursor: Point,
    packet: Point,
    packet_ordinal: u64,
    errors: [Option<ChecksumError>; MAX_ERRORS],
    error_count: u8,
    integrity: u8,
    recovery_ticks: u8,
    collected: u32,
    streak: u32,
    score: u64,
    queued_actions: Vec<GameAction>,
}

impl PacketSweep {
    #[must_use]
    pub fn new(seed: RunSeed) -> Self {
        let mut game = Self {
            seed,
            tick: SimulationTick(0),
            status: PacketSweepStatus::Running,
            cursor: START,
            packet: Point::new(1, 1),
            packet_ordinal: 0,
            errors: [None; MAX_ERRORS],
            error_count: 0,
            integrity: 3,
            recovery_ticks: 0,
            collected: 0,
            streak: 0,
            score: 0,
            queued_actions: Vec::with_capacity(4),
        };
        for _ in 0..3 {
            game.add_error();
        }
        game.packet = game.spawn_point(0x5041_434B_4554, game.packet_ordinal);
        game
    }

    pub fn reset(&mut self, seed: RunSeed) {
        *self = Self::new(seed);
    }

    pub fn handle_action(&mut self, action: GameAction) {
        if self.status == PacketSweepStatus::Running {
            self.queued_actions.push(action);
        }
    }

    pub fn update(&mut self, _step: SimulationStep) {
        if self.status != PacketSweepStatus::Running {
            self.queued_actions.clear();
            return;
        }

        self.tick = SimulationTick(self.tick.0.saturating_add(1));
        if self.tick.0 >= RUN_TICKS {
            self.status = PacketSweepStatus::Completed;
            self.queued_actions.clear();
            return;
        }
        self.recovery_ticks = self.recovery_ticks.saturating_sub(1);

        let actions = std::mem::take(&mut self.queued_actions);
        for action in actions {
            let heading = match action {
                GameAction::MoveUp => Some(Heading::Up),
                GameAction::MoveRight => Some(Heading::Right),
                GameAction::MoveDown => Some(Heading::Down),
                GameAction::MoveLeft => Some(Heading::Left),
                _ => None,
            };
            if let Some(heading) = heading {
                self.move_cursor(heading);
            }
        }
        self.queued_actions = Vec::with_capacity(4);

        if self.tick.0.is_multiple_of(ERROR_MOVE_INTERVAL) {
            self.move_errors();
        }
        self.check_collision();
    }

    pub fn set_paused(&mut self, paused: bool) {
        self.status = match (self.status, paused) {
            (PacketSweepStatus::Running, true) => PacketSweepStatus::Paused,
            (PacketSweepStatus::Paused, false) => PacketSweepStatus::Running,
            (status, _) => status,
        };
        if paused {
            self.queued_actions.clear();
        }
    }

    #[must_use]
    pub const fn status(&self) -> PacketSweepStatus {
        self.status
    }

    #[must_use]
    pub const fn game_status(&self) -> GameStatus {
        match self.status {
            PacketSweepStatus::Running | PacketSweepStatus::Paused => GameStatus::Running,
            PacketSweepStatus::Completed | PacketSweepStatus::Failed => GameStatus::Finished,
        }
    }

    #[must_use]
    pub fn result(&self) -> Option<GameResult> {
        self.game_status()
            .eq(&GameStatus::Finished)
            .then(|| GameResult {
                game_id: GameId::parse("packet-sweep").expect("static game identifier is valid"),
                mode_id: ModeId::parse("maintenance-run").expect("static mode identifier is valid"),
                rules_revision: rules_revision(),
                seed: self.seed,
                final_tick: self.tick,
                score: self.score,
                outcome: if self.status == PacketSweepStatus::Completed {
                    GameOutcome::Completed
                } else {
                    GameOutcome::GameOver
                },
                final_state_hash: self.state_hash(),
                discoveries: Vec::new(),
            })
    }

    #[must_use]
    pub const fn tick(&self) -> SimulationTick {
        self.tick
    }

    #[must_use]
    pub const fn cursor(&self) -> Point {
        self.cursor
    }

    #[must_use]
    pub const fn packet(&self) -> Point {
        self.packet
    }

    #[must_use]
    pub fn errors(&self) -> impl Iterator<Item = ChecksumError> + '_ {
        self.errors.iter().flatten().copied()
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
    pub const fn collected(&self) -> u32 {
        self.collected
    }

    #[must_use]
    pub const fn streak(&self) -> u32 {
        self.streak
    }

    #[must_use]
    pub const fn score(&self) -> u64 {
        self.score
    }

    #[must_use]
    pub const fn remaining_ticks(&self) -> u64 {
        RUN_TICKS.saturating_sub(self.tick.0)
    }

    #[must_use]
    pub fn state_hash(&self) -> StateHash {
        let mut hash = Fnv1a::new();
        hash.u16(rules_revision().get());
        hash.u64(self.seed.0);
        hash.u64(self.tick.0);
        hash.u8(self.status as u8);
        hash.point(self.cursor);
        hash.point(self.packet);
        hash.u64(self.packet_ordinal);
        hash.u8(self.error_count);
        for error in self.errors {
            hash.u8(u8::from(error.is_some()));
            if let Some(error) = error {
                hash.point(error.position);
                hash.u8(error.heading as u8);
            }
        }
        hash.u8(self.integrity);
        hash.u8(self.recovery_ticks);
        hash.u32(self.collected);
        hash.u32(self.streak);
        hash.u64(self.score);
        StateHash(hash.finish())
    }

    fn move_cursor(&mut self, heading: Heading) {
        let next = self.cursor.moved(heading);
        if is_open(next) {
            self.cursor = next;
        }
        self.check_collision();
        if self.cursor == self.packet {
            let points =
                100_u64.saturating_add(25_u64.saturating_mul(u64::from(self.streak.min(20))));
            self.score = self.score.saturating_add(points);
            self.streak = self.streak.saturating_add(1);
            self.collected = self.collected.saturating_add(1);
            if self.collected.is_multiple_of(15) && usize::from(self.error_count) < MAX_ERRORS {
                self.add_error();
            }
            self.packet_ordinal = self.packet_ordinal.wrapping_add(1);
            self.packet = self.spawn_point(0x5041_434B_4554, self.packet_ordinal);
        }
    }

    fn move_errors(&mut self) {
        for error in self.errors.iter_mut().flatten() {
            let next = error.position.moved(error.heading);
            if is_open(next) {
                error.position = next;
            } else {
                error.heading = error.heading.reflected();
                let reflected = error.position.moved(error.heading);
                if is_open(reflected) {
                    error.position = reflected;
                }
            }
        }
    }

    fn check_collision(&mut self) {
        if self.recovery_ticks == 0
            && self
                .errors
                .iter()
                .flatten()
                .any(|error| error.position == self.cursor)
        {
            self.integrity = self.integrity.saturating_sub(1);
            self.streak = 0;
            self.cursor = START;
            self.recovery_ticks = RECOVERY_TICKS;
            if self.integrity == 0 {
                self.status = PacketSweepStatus::Failed;
                self.queued_actions.clear();
            }
        }
    }

    fn add_error(&mut self) {
        if usize::from(self.error_count) >= MAX_ERRORS {
            return;
        }
        let ordinal = u64::from(self.error_count);
        let position = self.spawn_point(0x4552_524F_5253, ordinal);
        let heading = match deterministic_value(self.seed, 0x4845_4144_494E_47, ordinal) % 4 {
            0 => Heading::Up,
            1 => Heading::Right,
            2 => Heading::Down,
            _ => Heading::Left,
        };
        self.errors[usize::from(self.error_count)] = Some(ChecksumError { position, heading });
        self.error_count += 1;
    }

    fn spawn_point(&self, stream: u64, ordinal: u64) -> Point {
        let capacity = u64::try_from((ARENA_WIDTH - 2) * (ARENA_HEIGHT - 2))
            .expect("arena capacity is positive");
        for attempt in 0..capacity {
            let value = deterministic_value(self.seed, stream, ordinal.wrapping_add(attempt));
            let point = Point::new(
                1 + i16::try_from(value % u64::try_from(ARENA_WIDTH - 2).expect("width"))
                    .expect("x fits"),
                1 + i16::try_from(
                    (value / u64::try_from(ARENA_WIDTH - 2).expect("width"))
                        % u64::try_from(ARENA_HEIGHT - 2).expect("height"),
                )
                .expect("y fits"),
            );
            if is_open(point)
                && point != self.cursor
                && point != self.packet
                && !self
                    .errors
                    .iter()
                    .flatten()
                    .any(|error| error.position == point)
            {
                return point;
            }
        }
        Point::new(1, 1)
    }
}

#[must_use]
pub fn is_wall(point: Point) -> bool {
    point.x <= 0
        || point.y <= 0
        || point.x >= ARENA_WIDTH - 1
        || point.y >= ARENA_HEIGHT - 1
        || FIXED_WALLS.contains(&point)
}

fn is_open(point: Point) -> bool {
    !is_wall(point)
}

fn deterministic_value(seed: RunSeed, stream: u64, ordinal: u64) -> u64 {
    let mut value = seed.0 ^ stream ^ ordinal.wrapping_mul(0x9E37_79B9_7F4A_7C15);
    value = (value ^ (value >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    value = (value ^ (value >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    value ^ (value >> 31)
}

struct Fnv1a(u64);

impl Fnv1a {
    const fn new() -> Self {
        Self(14_695_981_039_346_656_037)
    }

    const fn finish(self) -> u64 {
        self.0
    }

    fn byte(&mut self, value: u8) {
        self.0 = (self.0 ^ u64::from(value)).wrapping_mul(1_099_511_628_211);
    }

    fn u8(&mut self, value: u8) {
        self.byte(value);
    }

    fn u16(&mut self, value: u16) {
        for byte in value.to_le_bytes() {
            self.byte(byte);
        }
    }

    fn u32(&mut self, value: u32) {
        for byte in value.to_le_bytes() {
            self.byte(byte);
        }
    }

    fn u64(&mut self, value: u64) {
        for byte in value.to_le_bytes() {
            self.byte(byte);
        }
    }

    fn point(&mut self, point: Point) {
        for byte in point
            .x
            .to_le_bytes()
            .into_iter()
            .chain(point.y.to_le_bytes())
        {
            self.byte(byte);
        }
    }
}

#[cfg(test)]
mod tests {
    use raster_display::DisplayBuffer;

    use super::*;

    fn step(game: &mut PacketSweep, tick: u64, actions: &[GameAction]) {
        for action in actions {
            game.handle_action(*action);
        }
        game.update(SimulationStep {
            tick: SimulationTick(tick),
        });
    }

    #[test]
    fn same_seed_has_same_initial_layout_and_hash() {
        let first = PacketSweep::new(RunSeed(90));
        let second = PacketSweep::new(RunSeed(90));
        assert_eq!(first.packet(), second.packet());
        assert_eq!(
            first.errors().collect::<Vec<_>>(),
            second.errors().collect::<Vec<_>>()
        );
        assert_eq!(first.state_hash(), second.state_hash());
        assert_eq!(first.errors().count(), 3);
    }

    #[test]
    fn cursor_respects_fixed_walls() {
        let mut game = PacketSweep::new(RunSeed(1));
        game.cursor = Point::new(1, 1);
        step(&mut game, 1, &[GameAction::MoveLeft, GameAction::MoveUp]);
        assert_eq!(game.cursor(), Point::new(1, 1));
        game.cursor = Point::new(5, 3);
        step(&mut game, 2, &[GameAction::MoveDown]);
        assert_eq!(game.cursor(), Point::new(5, 3));
    }

    #[test]
    fn packets_score_from_current_streak_and_add_errors() {
        let mut game = PacketSweep::new(RunSeed(2));
        game.errors.fill(None);
        game.error_count = 3;
        game.collected = 14;
        game.streak = 20;
        game.packet = game.cursor.moved(Heading::Right);

        step(&mut game, 1, &[GameAction::MoveRight]);

        assert_eq!(game.score(), 600);
        assert_eq!(game.streak(), 21);
        assert_eq!(game.collected(), 15);
        assert_eq!(game.errors().count(), 1);
        assert_eq!(game.error_count, 4);
    }

    #[test]
    fn collision_damages_resets_and_protects() {
        let mut game = PacketSweep::new(RunSeed(3));
        game.errors.fill(None);
        game.errors[0] = Some(ChecksumError {
            position: game.cursor.moved(Heading::Right),
            heading: Heading::Right,
        });
        game.streak = 7;

        step(&mut game, 1, &[GameAction::MoveRight]);

        assert_eq!(game.integrity(), 2);
        assert_eq!(game.cursor(), START);
        assert_eq!(game.streak(), 0);
        assert_eq!(game.recovery_ticks(), RECOVERY_TICKS);
        game.errors[0] = Some(ChecksumError {
            position: START,
            heading: Heading::Right,
        });
        step(&mut game, 2, &[]);
        assert_eq!(game.integrity(), 2);
    }

    #[test]
    fn zero_integrity_fails_and_timer_expiry_completes() {
        let mut failed = PacketSweep::new(RunSeed(4));
        failed.errors.fill(None);
        failed.integrity = 1;
        failed.errors[0] = Some(ChecksumError {
            position: START,
            heading: Heading::Right,
        });
        failed.check_collision();
        assert_eq!(failed.status(), PacketSweepStatus::Failed);
        assert_eq!(
            failed.result().expect("failure result").outcome,
            GameOutcome::GameOver
        );

        let mut completed = PacketSweep::new(RunSeed(4));
        completed.tick = SimulationTick(RUN_TICKS - 1);
        step(&mut completed, RUN_TICKS, &[]);
        assert_eq!(completed.status(), PacketSweepStatus::Completed);
        assert_eq!(
            completed.result().expect("completion result").outcome,
            GameOutcome::Completed
        );
    }

    #[test]
    fn seeded_errors_reflect_at_walls() {
        let mut game = PacketSweep::new(RunSeed(5));
        game.errors.fill(None);
        game.errors[0] = Some(ChecksumError {
            position: Point::new(1, 2),
            heading: Heading::Left,
        });
        game.move_errors();
        assert_eq!(
            game.errors[0],
            Some(ChecksumError {
                position: Point::new(2, 2),
                heading: Heading::Right,
            })
        );
    }

    #[test]
    fn pause_freezes_authoritative_state() {
        let mut game = PacketSweep::new(RunSeed(6));
        game.set_paused(true);
        let before = game.state_hash();
        step(&mut game, 1, &[GameAction::MoveRight]);
        assert_eq!(game.state_hash(), before);
        game.set_paused(false);
        step(&mut game, 2, &[GameAction::MoveRight]);
        assert_ne!(game.state_hash(), before);
    }

    #[test]
    fn golden_run_is_deterministic() {
        let mut first = PacketSweep::new(RunSeed(0x5452_4143_4539_30));
        let mut second = first.clone();
        for tick in 1..=900 {
            let actions: &[GameAction] = match tick % 37 {
                0 => &[GameAction::MoveRight],
                9 => &[GameAction::MoveDown],
                18 => &[GameAction::MoveLeft],
                27 => &[GameAction::MoveUp],
                _ => &[],
            };
            step(&mut first, tick, actions);
            step(&mut second, tick, actions);
        }
        assert_eq!(first.state_hash(), second.state_hash());
        assert_eq!(first.score(), second.score());
        assert_eq!(first.state_hash(), StateHash(3_340_492_426_130_267_100));
    }

    #[test]
    fn structured_snapshots_cover_key_states() {
        let game = PacketSweep::new(RunSeed(7));
        let mut display = DisplayBuffer::canonical();
        render(&game, &mut display).expect("initial render");
        let initial = display.snapshot().character_grid();
        assert!(initial.contains("PACKET SWEEP"));
        assert!(initial.contains("INTEGRITY 3"));

        let mut danger = game.clone();
        danger.integrity = 1;
        danger.recovery_ticks = 30;
        render(&danger, &mut display).expect("danger render");
        let danger_grid = display.snapshot().character_grid();
        assert!(danger_grid.contains("RECOVERY"));
        assert!(danger_grid.contains("INTEGRITY 1"));

        let mut paused = game.clone();
        paused.set_paused(true);
        render(&paused, &mut display).expect("paused render");
        assert!(display.snapshot().character_grid().contains("PAUSED"));

        let mut failed = game;
        failed.status = PacketSweepStatus::Failed;
        render(&failed, &mut display).expect("failed render");
        assert!(display.snapshot().character_grid().contains("GAME OVER"));
    }
}

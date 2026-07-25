// SPDX-License-Identifier: MPL-2.0

use std::time::Duration;

use crate::{SimulationStep, SimulationTick};

/// Canonical authoritative simulation frequency.
pub const SIMULATION_HZ: u32 = 60;

/// Largest host-frame delta accepted before catch-up is clamped.
pub const MAX_FRAME_DELTA: Duration = Duration::from_millis(250);

const NANOS_PER_SECOND: u128 = 1_000_000_000;

/// Converts host elapsed time into deterministic fixed simulation steps.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct FixedStepClock {
    current_tick: SimulationTick,
    scaled_accumulator: u128,
}

impl FixedStepClock {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            current_tick: SimulationTick(0),
            scaled_accumulator: 0,
        }
    }

    #[must_use]
    pub const fn current_tick(&self) -> SimulationTick {
        self.current_tick
    }

    /// Records elapsed host time and returns the authoritative steps now due.
    ///
    /// Pausing discards elapsed and partial accumulated time so resume never
    /// catches up time that passed while simulation was suspended.
    pub fn advance(&mut self, elapsed: Duration, paused: bool) -> StepBatch {
        if paused {
            self.scaled_accumulator = 0;
            return StepBatch::empty(self.current_tick.next());
        }

        let elapsed = elapsed.min(MAX_FRAME_DELTA);
        self.scaled_accumulator = self
            .scaled_accumulator
            .saturating_add(elapsed.as_nanos().saturating_mul(u128::from(SIMULATION_HZ)));
        let count = self.scaled_accumulator / NANOS_PER_SECOND;
        self.scaled_accumulator %= NANOS_PER_SECOND;

        let count = u32::try_from(count).unwrap_or(u32::MAX);
        let first_tick = self.current_tick.next();
        self.current_tick.0 = self.current_tick.0.saturating_add(u64::from(count));

        StepBatch { first_tick, count }
    }
}

/// Consecutive fixed steps due after one host-frame update.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct StepBatch {
    first_tick: SimulationTick,
    count: u32,
}

impl StepBatch {
    const fn empty(first_tick: SimulationTick) -> Self {
        Self {
            first_tick,
            count: 0,
        }
    }

    #[must_use]
    pub const fn len(self) -> u32 {
        self.count
    }

    #[must_use]
    pub const fn is_empty(self) -> bool {
        self.count == 0
    }

    pub fn iter(self) -> impl ExactSizeIterator<Item = SimulationStep> {
        (0..self.count).map(move |offset| SimulationStep {
            tick: SimulationTick(self.first_tick.0.saturating_add(u64::from(offset))),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn single_long_frame_is_clamped() {
        let mut clock = FixedStepClock::new();

        let batch = clock.advance(Duration::from_secs(1), false);

        // The stall clamp intentionally limits a single one-second frame.
        assert_eq!(batch.len(), 15);
    }

    #[test]
    fn normal_frame_chunks_do_not_accumulate_rounding_error() {
        let mut clock = FixedStepClock::new();
        let mut count = 0;

        for _ in 0..100 {
            count += clock.advance(Duration::from_millis(10), false).len();
        }

        assert_eq!(count, 60);
        assert_eq!(clock.current_tick(), SimulationTick(60));
    }

    #[test]
    fn long_stalls_are_clamped() {
        let mut clock = FixedStepClock::new();

        assert_eq!(
            clock.advance(Duration::from_secs(10), false).len(),
            SIMULATION_HZ / 4
        );
    }

    #[test]
    fn pause_discards_partial_time_without_catch_up() {
        let mut clock = FixedStepClock::new();
        assert!(clock.advance(Duration::from_millis(10), false).is_empty());
        assert!(clock.advance(Duration::from_secs(5), true).is_empty());

        assert!(clock.advance(Duration::from_millis(10), false).is_empty());
        assert_eq!(clock.current_tick(), SimulationTick(0));
    }

    #[test]
    fn batches_report_consecutive_ticks() {
        let mut clock = FixedStepClock::new();
        let ticks = clock
            .advance(Duration::from_millis(50), false)
            .iter()
            .map(|step| step.tick)
            .collect::<Vec<_>>();

        assert_eq!(
            ticks,
            vec![SimulationTick(1), SimulationTick(2), SimulationTick(3)]
        );
    }
}

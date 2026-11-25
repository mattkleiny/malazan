//! The simulation clock: turns frame time into fixed steps.

use surreal::ecs::Component;

/// Length of one fixed tick (s).
pub const FIXED_STEP: f64 = 1.0 / 30.0;

/// Accumulates frame time and hands it out one fixed step at a time.
#[derive(Component, Default)]
pub struct SimulationClock {
  owed: f64, // Time fed but not yet handed out (s).
}

impl SimulationClock {
  /// Adds a frame's worth of time to the clock.
  pub fn feed(&mut self, delta: f64) {
    self.owed += delta.max(0.0);
  }

  /// Takes one fixed step, if a whole one is owed.
  pub fn take_tick(&mut self) -> Option<f64> {
    if self.owed < FIXED_STEP {
      return None;
    }

    self.owed -= FIXED_STEP;

    Some(FIXED_STEP)
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn hands_out_whole_steps_and_keeps_the_rest() {
    let mut clock = SimulationClock::default();

    clock.feed(FIXED_STEP * 2.5);

    assert_eq!(clock.take_tick(), Some(FIXED_STEP));
    assert_eq!(clock.take_tick(), Some(FIXED_STEP));
    assert_eq!(clock.take_tick(), None);
  }
}

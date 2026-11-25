//! Foundations shared by every other crate in the game.

use surreal::ecs::{TickEvent, World};

mod clock;

pub use clock::{FIXED_STEP, SimulationClock};

/// Sent to the game every frame.
#[derive(TickEvent, Clone)]
pub struct VariableTick {
  pub delta: f64,
}

/// Sent to the game once per due step of the [SimulationClock].
#[derive(TickEvent, Clone)]
pub struct FixedTick {
  pub delta: f64,
}

/// Installs the common systems into the given [World].
pub fn install_common_systems(world: &mut World) {
  world.add_global(SimulationClock::default());
}

surreal::ecs::install_systems!("common", order: 100, install: install_common_systems);

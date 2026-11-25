//! Map modes: the ways the map's locations can be coloured.

use surreal::utilities::inventory;

use crate::map::Map;

pub mod standard;

/// The colour a map mode fills a location with.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Fill {
  pub color: [u8; 3], // sRGB.
  pub strength: u8,   // How much of the ground it covers, up to 255 for all of it.
}

impl Fill {
  /// No fill: the ground or the sea shows as it is.
  pub const CLEAR: Fill = Fill {
    color: [0, 0, 0],
    strength: 0,
  };

  pub fn solid(color: [u8; 3]) -> Fill {
    Fill {
      color,
      strength: u8::MAX,
    }
  }
}

/// One way of colouring the map, such as by terrain or by province.
pub trait MapMode: Sync + 'static {
  /// The name the mode is chosen by.
  fn key(&self) -> &'static str;

  /// The name players see.
  fn label(&self) -> &'static str;

  /// The fill of every location, in the order of the map's table of them.
  fn fills(&self, map: &Map) -> Vec<Fill>;
}

/// A map mode as [map_mode!] submits it.
#[doc(hidden)]
pub struct MapModeRegistration {
  pub order: i32,
  pub mode: &'static dyn MapMode,
}

inventory::collect!(MapModeRegistration);

/// Registers a map mode, and where it sits among the others. The calling
/// crate must depend on `surreal`.
///
/// ```ignore
/// map_mode!(order: 100, WealthMode);
/// ```
#[macro_export]
macro_rules! map_mode {
  (order: $order:expr, $mode:expr $(,)?) => {
    ::surreal::utilities::inventory::submit! {
      $crate::MapModeRegistration {
        order: $order,
        mode: &$mode,
      }
    }
  };
}

/// Every registered map mode, lowest order first and then by key.
pub fn map_modes() -> Vec<&'static dyn MapMode> {
  let mut registrations: Vec<_> = inventory::iter::<MapModeRegistration>().collect();

  registrations.sort_by_key(|registration| (registration.order, registration.mode.key()));
  registrations
    .into_iter()
    .map(|registration| registration.mode)
    .collect()
}

/// The registered map mode chosen by `key`.
pub fn map_mode(key: &str) -> Option<&'static dyn MapMode> {
  inventory::iter::<MapModeRegistration>()
    .map(|registration| registration.mode)
    .find(|mode| mode.key() == key)
}

#[cfg(test)]
mod tests {
  use super::*;

  struct TestMode;

  impl MapMode for TestMode {
    fn key(&self) -> &'static str {
      "test"
    }

    fn label(&self) -> &'static str {
      "Test"
    }

    fn fills(&self, map: &Map) -> Vec<Fill> {
      vec![Fill::CLEAR; map.locations.len()]
    }
  }

  crate::map_mode!(order: -1, TestMode);

  #[test]
  fn a_registered_mode_is_found_by_its_key() {
    assert_eq!(map_mode("test").map(|mode| mode.label()), Some("Test"));
    assert!(map_mode("no such mode").is_none());
  }

  #[test]
  fn modes_are_listed_in_order() {
    let keys: Vec<&str> = map_modes().iter().map(|mode| mode.key()).collect();

    assert_eq!(keys, [
      "test",
      "terrain",
      "locations",
      "provinces",
      "areas",
      "regions",
      "continents"
    ]);
  }
}

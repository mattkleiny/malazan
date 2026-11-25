//! Selections on the terrain: what the cursor rests on and what was chosen,
//! as the shown map mode sees them.

use game_map::{LocationId, Map, MapMode, Selection};

/// A location the player picked, and what it selects in the shown map mode.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Picked {
  pub location: Option<LocationId>,
  pub selection: Option<Selection>,
}

impl Picked {
  /// Picks a location afresh; whether what it selects changed.
  fn pick(&mut self, map: &Map, mode: &dyn MapMode, location: Option<LocationId>) -> bool {
    self.location = location;
    self.refresh(map, mode)
  }

  /// Works out the selection again for the same location; whether it changed.
  fn refresh(&mut self, map: &Map, mode: &dyn MapMode) -> bool {
    let selection = self.location.and_then(|location| mode.select(map, location));
    let changed = match (&self.selection, &selection) {
      (Some(before), Some(after)) => !before.is_same(after),
      (None, None) => false,
      _ => true,
    };

    self.selection = selection;
    changed
  }
}

/// What the cursor rests on and what the player chose.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Selections {
  pub hovered: Picked,
  pub selected: Picked,
}

impl Selections {
  /// Rests the cursor on a location, or on nothing; whether the hover changed.
  pub fn hover(&mut self, map: &Map, mode: &dyn MapMode, location: Option<LocationId>) -> bool {
    self.hovered.pick(map, mode, location)
  }

  /// Chooses what a location selects, or clears the choice; whether it changed.
  pub fn select(&mut self, map: &Map, mode: &dyn MapMode, location: Option<LocationId>) -> bool {
    self.selected.pick(map, mode, location)
  }

  /// Selects again from the same locations once the map mode has changed, so
  /// a location chosen in province mode becomes its region in region mode.
  pub fn refresh(&mut self, map: &Map, mode: &dyn MapMode) -> bool {
    let hovered = self.hovered.refresh(map, mode);
    let selected = self.selected.refresh(map, mode);

    hovered || selected
  }

  /// For each location, whether it is lit as hovered and as selected.
  pub fn highlights(&self, location_count: usize) -> Vec<[bool; 2]> {
    let mut lit = vec![[false; 2]; location_count];

    for (slot, picked) in [&self.hovered, &self.selected].into_iter().enumerate() {
      let Some(selection) = &picked.selection else {
        continue;
      };

      for location in &selection.locations {
        if let Some(entry) = lit.get_mut(location.index()) {
          entry[slot] = true;
        }
      }
    }

    lit
  }
}

#[cfg(test)]
mod tests {
  use game_map::{Fill, Heightfield, MapParts, Tier, division_selection};

  use super::*;

  /// Selects whole provinces.
  struct ProvinceTestMode;

  impl MapMode for ProvinceTestMode {
    fn key(&self) -> &'static str {
      "province_test"
    }

    fn label(&self) -> &'static str {
      "Provinces"
    }

    fn fills(&self, map: &Map) -> Vec<Fill> {
      vec![Fill::CLEAR; map.locations.len()]
    }

    fn select(&self, map: &Map, location: LocationId) -> Option<Selection> {
      division_selection(map, location, Tier::Province)
    }
  }

  /// Selects single locations, as every mode does unless it says otherwise.
  struct LocationTestMode;

  impl MapMode for LocationTestMode {
    fn key(&self) -> &'static str {
      "location_test"
    }

    fn label(&self) -> &'static str {
      "Locations"
    }

    fn fills(&self, map: &Map) -> Vec<Fill> {
      vec![Fill::CLEAR; map.locations.len()]
    }
  }

  const A: LocationId = LocationId(0);
  const B: LocationId = LocationId(1);
  const C: LocationId = LocationId(2);

  /// A and B share a province; C has its own.
  fn map() -> Map {
    Map::from_parts(MapParts {
      definitions: "1,0,0,A,P1,Ar,R,C,plains\n2,0,0,B,P1,Ar,R,C,plains\n3,0,0,C,P2,Ar,R,C,plains",
      width: 3,
      height: 1,
      key_colors: &[[1, 0, 0], [2, 0, 0], [3, 0, 0]],
      heights: Heightfield::from_samples(1, 1, vec![0]).expect("one sample fills a one by one grid"),
      sea_level: 0.5,
    })
    .unwrap()
  }

  #[test]
  fn hovering_a_location_lights_everything_the_mode_selects() {
    let mut selections = Selections::default();

    selections.hover(&map(), &ProvinceTestMode, Some(A));

    assert_eq!(selections.highlights(3), [[true, false], [true, false], [false, false]]);
  }

  #[test]
  fn moving_within_the_same_province_is_no_change() {
    let map = map();
    let mut selections = Selections::default();
    selections.hover(&map, &ProvinceTestMode, Some(A));

    assert!(!selections.hover(&map, &ProvinceTestMode, Some(B)));
    assert!(selections.hover(&map, &ProvinceTestMode, Some(C)));
  }

  #[test]
  fn changing_mode_selects_again_from_the_same_location() {
    let map = map();
    let mut selections = Selections::default();
    selections.select(&map, &LocationTestMode, Some(A));

    let changed = selections.refresh(&map, &ProvinceTestMode);

    assert!(changed);
    assert_eq!(selections.highlights(3), [[false, true], [false, true], [false, false]]);
  }

  #[test]
  fn selecting_nothing_clears_the_choice() {
    let map = map();
    let mut selections = Selections::default();
    selections.select(&map, &LocationTestMode, Some(C));

    assert!(selections.select(&map, &LocationTestMode, None));
    assert!(selections.selected.selection.is_none());
  }
}

//! Patches: blocks of the map as they stood, kept so a stroke can be undone
//! and done again.

use crate::{
  editing::{CellRect, widen},
  locations::LocationId,
  map::Map,
};

/// Blocks of the location raster and the heightfield as they stood.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct MapPatch {
  cells: Option<(CellRect, Vec<LocationId>)>,
  heights: Option<(CellRect, Vec<u16>)>,
}

impl MapPatch {
  /// The raster cells the patch covers.
  pub fn cells_rect(&self) -> Option<CellRect> {
    self.cells.as_ref().map(|(rect, _)| *rect)
  }

  /// The heightfield samples the patch covers.
  pub fn heights_rect(&self) -> Option<CellRect> {
    self.heights.as_ref().map(|(rect, _)| *rect)
  }
}

/// A stroke's change, kept both ways.
#[derive(Clone, Debug, PartialEq)]
pub struct MapEdit {
  pub before: MapPatch,
  pub after: MapPatch,
}

impl Map {
  /// Writes a patch back over the map. The caller refreshes adjacency when it
  /// touches the raster.
  pub fn apply_patch(&mut self, patch: &MapPatch) {
    if let Some((rect, cells)) = &patch.cells {
      for ((column, row), &location) in rect.cells().zip(cells) {
        self.raster.set_cell(i64::from(column), i64::from(row), location);
      }
    }

    if let Some((rect, samples)) = &patch.heights {
      for ((column, row), &sample) in rect.cells().zip(samples) {
        self.heights.set_sample(
          i64::from(column),
          i64::from(row),
          f64::from(sample) / f64::from(u16::MAX),
        );
      }
    }
  }

  fn patch_of(&self, cells: Option<CellRect>, heights: Option<CellRect>) -> MapPatch {
    MapPatch {
      cells: cells.map(|rect| (rect, block(self.raster.cells(), self.raster.width(), rect))),
      heights: heights.map(|rect| (rect, block(self.heights.samples(), self.heights.width(), rect))),
    }
  }
}

/// One stroke of a brush, from the press to the release: it keeps the map as
/// it stood when the stroke began, and where the stroke has been.
pub struct Stroke {
  cells: Vec<LocationId>,
  heights: Vec<u16>,
  cells_touched: Option<CellRect>,
  heights_touched: Option<CellRect>,
}

impl Stroke {
  pub fn begin(map: &Map) -> Stroke {
    Stroke {
      cells: map.raster.cells().to_vec(),
      heights: map.heights.samples().to_vec(),
      cells_touched: None,
      heights_touched: None,
    }
  }

  pub fn touch_cells(&mut self, rect: Option<CellRect>) {
    self.cells_touched = widen(self.cells_touched, rect);
  }

  pub fn touch_heights(&mut self, rect: Option<CellRect>) {
    self.heights_touched = widen(self.heights_touched, rect);
  }

  /// The stroke's change, or none when it changed nothing.
  pub fn finish(self, map: &Map) -> Option<MapEdit> {
    if self.cells_touched.is_none() && self.heights_touched.is_none() {
      return None;
    }

    let before = MapPatch {
      cells: self
        .cells_touched
        .map(|rect| (rect, block(&self.cells, map.raster.width(), rect))),
      heights: self
        .heights_touched
        .map(|rect| (rect, block(&self.heights, map.heights.width(), rect))),
    };

    Some(MapEdit {
      before,
      after: map.patch_of(self.cells_touched, self.heights_touched),
    })
  }
}

/// The values of a block of a grid stored row by row.
fn block<T: Copy>(values: &[T], grid_width: u32, rect: CellRect) -> Vec<T> {
  rect
    .cells()
    .map(|(column, row)| values[row as usize * grid_width as usize + column as usize])
    .collect()
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::{
    editing::brushes::{Brush, Sculpt, paint_location, sculpt},
    heights::Heightfield,
    map::{MapParts, MapPosition},
  };

  fn map() -> Map {
    Map::from_parts(MapParts {
      definitions: "1,0,0,Land,P,A,R,C,plains\n2,0,0,Bay,Q,B,S,D,sea",
      width: 8,
      height: 8,
      key_colors: &[[1, 0, 0]; 64],
      heights: Heightfield::from_samples(8, 8, vec![u16::MAX / 2; 64]).unwrap(),
      sea_level: 0.3,
    })
    .unwrap()
  }

  fn brush() -> Brush {
    Brush {
      centre: MapPosition { x: 4.0, y: 4.0 },
      radius: 2.0,
      strength: 1.0,
    }
  }

  #[test]
  fn undoing_a_stroke_puts_the_map_back() {
    let mut map = map();
    let original = map.clone();
    let mut stroke = Stroke::begin(&map);
    let painted = paint_location(&mut map, &brush(), LocationId(1));
    stroke.touch_cells(painted);
    let raised = sculpt(&mut map, &brush(), Sculpt::Raise);
    stroke.touch_heights(raised);
    let edit = stroke.finish(&map).unwrap();

    map.apply_patch(&edit.before);

    assert_eq!(map.raster, original.raster);
    assert_eq!(map.heights, original.heights);
  }

  #[test]
  fn redoing_a_stroke_makes_the_change_again() {
    let mut map = map();
    let mut stroke = Stroke::begin(&map);
    let painted = paint_location(&mut map, &brush(), LocationId(1));
    stroke.touch_cells(painted);
    let changed = map.clone();
    let edit = stroke.finish(&map).unwrap();
    map.apply_patch(&edit.before);

    map.apply_patch(&edit.after);

    assert_eq!(map.raster, changed.raster);
  }

  #[test]
  fn a_stroke_that_changed_nothing_leaves_no_edit() {
    let map = map();

    assert_eq!(Stroke::begin(&map).finish(&map), None);
  }

  #[test]
  fn a_patch_covers_only_what_the_stroke_touched() {
    let mut map = map();
    let mut stroke = Stroke::begin(&map);
    let painted = paint_location(&mut map, &brush(), LocationId(1));
    stroke.touch_cells(painted);

    let edit = stroke.finish(&map).unwrap();

    assert_eq!(edit.after.cells_rect(), painted);
    assert_eq!(edit.after.heights_rect(), None);
  }
}

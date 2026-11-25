//! Brushes: painting locations and shaping the ground.

use crate::{
  editing::CellRect,
  locations::LocationId,
  map::{Map, MapPosition},
};

/// How much a full-strength raise or lower moves the ground at the brush's
/// centre in one dab, as a share of the heightfield's range.
const SCULPT_RATE: f64 = 0.02;

/// Where and how hard one dab of a brush lands.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Brush {
  pub centre: MapPosition,
  pub radius: f64,   // (map units)
  pub strength: f64, // 0..1
}

/// What a dab does to the ground.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Sculpt {
  Raise,
  Lower,
  Smooth,
  Flatten { height: f64 }, // Height levelled towards (0..1).
}

/// Paints every cell within the brush with a location; the cells that
/// changed, if any.
pub fn paint_location(map: &mut Map, brush: &Brush, location: LocationId) -> Option<CellRect> {
  if location.index() >= map.locations.len() {
    return None;
  }

  // Even the smallest brush paints the cell it lands in.
  let radius = brush.radius.max(0.5);
  let raster = &mut map.raster;
  let reach = CellRect::around(brush.centre.x, brush.centre.y, radius, raster.width(), raster.height())?;
  let mut changed: Option<CellRect> = None;

  for (column, row) in reach.cells() {
    let (column, row) = (i64::from(column), i64::from(row));
    let distance = (column as f64 + 0.5 - brush.centre.x).hypot(row as f64 + 0.5 - brush.centre.y);

    if distance > radius || raster.cell(column, row) == Some(location) {
      continue;
    }

    raster.set_cell(column, row, location);
    changed = super::widen(changed, Some(cell(column, row)));
  }

  changed
}

/// Shapes the ground under the brush, strongest at its centre; the
/// heightfield samples that changed, if any.
pub fn sculpt(map: &mut Map, brush: &Brush, sculpt: Sculpt) -> Option<CellRect> {
  let heights = &map.heights;
  let scale_x = f64::from(heights.width()) / map.raster.width().max(1) as f64;
  let scale_y = f64::from(heights.height()) / map.raster.height().max(1) as f64;
  let (centre_x, centre_y) = (brush.centre.x * scale_x, brush.centre.y * scale_y);
  let radius = (brush.radius * scale_x).max(0.5);
  let reach = CellRect::around(centre_x, centre_y, radius, heights.width(), heights.height())?;

  // Worked out before any is written, so smoothing reads the ground as it was.
  let mut shaped = Vec::with_capacity(reach.area());

  for (column, row) in reach.cells() {
    let (column, row) = (i64::from(column), i64::from(row));
    let distance = (column as f64 + 0.5 - centre_x).hypot(row as f64 + 0.5 - centre_y);
    let weight = falloff(distance / radius) * brush.strength.clamp(0.0, 1.0);

    if weight <= 0.0 {
      continue;
    }

    let height = heights.sample_at(column, row);
    let target = match sculpt {
      Sculpt::Raise => height + SCULPT_RATE * weight,
      Sculpt::Lower => height - SCULPT_RATE * weight,
      Sculpt::Smooth => lerp(height, neighbourhood_mean(map, column, row), weight),
      Sculpt::Flatten { height: level } => lerp(height, level, weight),
    };

    shaped.push((column, row, target));
  }

  let mut changed: Option<CellRect> = None;

  for (column, row, height) in shaped {
    let before = map.heights.samples()[row as usize * map.heights.width() as usize + column as usize];
    map.heights.set_sample(column, row, height);

    if map.heights.samples()[row as usize * map.heights.width() as usize + column as usize] != before {
      changed = super::widen(changed, Some(cell(column, row)));
    }
  }

  changed
}

fn neighbourhood_mean(map: &Map, column: i64, row: i64) -> f64 {
  let mut total = 0.0;

  for offset_row in -1..=1 {
    for offset_column in -1..=1 {
      total += map.heights.sample_at(column + offset_column, row + offset_row);
    }
  }

  total / 9.0
}

/// How strongly a brush acts at a share of its radius from the centre: fully
/// at the centre, easing to nothing at the rim.
fn falloff(share: f64) -> f64 {
  let amount = (1.0 - share).clamp(0.0, 1.0);

  amount * amount * (3.0 - 2.0 * amount)
}

fn lerp(from: f64, to: f64, amount: f64) -> f64 {
  from + (to - from) * amount
}

fn cell(column: i64, row: i64) -> CellRect {
  CellRect {
    column: column as u32,
    row: row as u32,
    width: 1,
    height: 1,
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::{heights::Heightfield, map::MapParts};

  const LAND: LocationId = LocationId(0);
  const BAY: LocationId = LocationId(1);

  /// Eight by eight cells of land, with heights at the same resolution, all at
  /// half height.
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

  fn brush(x: f64, y: f64, radius: f64) -> Brush {
    Brush {
      centre: MapPosition { x, y },
      radius,
      strength: 1.0,
    }
  }

  #[test]
  fn painting_fills_the_cells_within_the_brush() {
    let mut map = map();

    let changed = paint_location(&mut map, &brush(4.0, 4.0, 1.0), BAY).unwrap();

    assert_eq!(changed, CellRect {
      column: 3,
      row: 3,
      width: 2,
      height: 2
    });
    assert_eq!(map.location_at(MapPosition { x: 3.5, y: 4.5 }), Some(BAY));
    assert_eq!(map.location_at(MapPosition { x: 5.5, y: 4.5 }), Some(LAND));
  }

  #[test]
  fn the_smallest_brush_paints_the_cell_it_lands_in() {
    let mut map = map();

    paint_location(&mut map, &brush(6.2, 1.9, 0.0), BAY);

    assert_eq!(map.location_at(MapPosition { x: 6.5, y: 1.5 }), Some(BAY));
  }

  #[test]
  fn painting_what_is_already_there_changes_nothing() {
    let mut map = map();

    assert_eq!(paint_location(&mut map, &brush(4.0, 4.0, 2.0), LAND), None);
  }

  #[test]
  fn a_location_the_map_does_not_have_cannot_be_painted() {
    let mut map = map();

    assert_eq!(paint_location(&mut map, &brush(4.0, 4.0, 2.0), LocationId(7)), None);
  }

  #[test]
  fn raising_lifts_the_centre_most() {
    let mut map = map();

    sculpt(&mut map, &brush(4.0, 4.0, 3.0), Sculpt::Raise);

    let centre = map.heights.sample_at(3, 3);
    let rim = map.heights.sample_at(1, 3);
    assert!(centre > rim && rim >= 0.5 - 1e-4, "centre {centre}, rim {rim}");
  }

  #[test]
  fn lowering_sinks_the_ground() {
    let mut map = map();

    sculpt(&mut map, &brush(4.0, 4.0, 3.0), Sculpt::Lower);

    assert!(map.heights.sample_at(3, 3) < 0.49);
  }

  #[test]
  fn flattening_levels_towards_a_height() {
    let mut map = map();

    sculpt(&mut map, &brush(4.0, 4.0, 3.0), Sculpt::Flatten { height: 0.9 });

    assert!(map.heights.sample_at(3, 3) > 0.8);
  }

  #[test]
  fn smoothing_wears_a_spike_down() {
    let mut map = map();
    map.heights.set_sample(4, 4, 1.0);

    sculpt(&mut map, &brush(4.5, 4.5, 2.0), Sculpt::Smooth);

    assert!(map.heights.sample_at(4, 4) < 0.7);
  }

  #[test]
  fn smoothing_flat_ground_changes_nothing() {
    let mut map = map();

    assert_eq!(sculpt(&mut map, &brush(4.0, 4.0, 2.0), Sculpt::Smooth), None);
  }

  #[test]
  fn a_brush_off_the_map_shapes_nothing() {
    let mut map = map();

    assert_eq!(sculpt(&mut map, &brush(-10.0, 4.0, 2.0), Sculpt::Raise), None);
  }
}

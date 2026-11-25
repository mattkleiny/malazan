//! The location raster: which location covers each point of the map.

use std::collections::HashMap;

use crate::{
  locations::{Location, LocationId},
  map::{MapError, MapPosition},
};

/// How many unknown colours one error lists.
const MAX_REPORTED_COLORS: usize = 16;

/// A grid over the map naming the location under each cell. One cell is one
/// map unit, and the grid wraps east to west.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LocationRaster {
  width: u32,
  height: u32,
  cells: Vec<LocationId>,
}

impl LocationRaster {
  /// Reads the raster from an image painted in the locations' key colours.
  pub fn from_key_colors(
    width: u32,
    height: u32,
    pixels: &[[u8; 3]],
    locations: &[Location],
  ) -> Result<Self, MapError> {
    if width == 0 || height == 0 || pixels.len() != width as usize * height as usize {
      return Err(MapError::UnreadableImage { image: "locations" });
    }

    let by_color: HashMap<[u8; 3], LocationId> = locations
      .iter()
      .enumerate()
      .map(|(index, location)| (location.key_color, LocationId(index as u16)))
      .collect();

    let mut cells = Vec::with_capacity(pixels.len());
    let mut unknown = Vec::new();

    for color in pixels {
      match by_color.get(color) {
        Some(&location) => cells.push(location),
        None if unknown.len() < MAX_REPORTED_COLORS && !unknown.contains(color) => unknown.push(*color),
        None => {}
      }
    }

    if !unknown.is_empty() {
      unknown.sort_unstable();

      return Err(MapError::UnknownColors { colors: unknown });
    }

    Ok(Self { width, height, cells })
  }

  pub fn width(&self) -> u32 {
    self.width
  }

  pub fn height(&self) -> u32 {
    self.height
  }

  /// Every cell's location, row by row from the north-west corner.
  pub fn cells(&self) -> &[LocationId] {
    &self.cells
  }

  /// The location of a cell; columns wrap east to west.
  pub fn cell(&self, column: i64, row: i64) -> Option<LocationId> {
    if row < 0 || row >= i64::from(self.height) {
      return None;
    }

    let column = column.rem_euclid(i64::from(self.width));

    self
      .cells
      .get(row as usize * self.width as usize + column as usize)
      .copied()
  }

  /// The location under a point of the map.
  pub fn location_at(&self, position: MapPosition) -> Option<LocationId> {
    if !position.x.is_finite() || !position.y.is_finite() {
      return None;
    }

    self.cell(position.x.floor() as i64, position.y.floor() as i64)
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::{divisions::ProvinceId, locations::Terrain};

  const RED: [u8; 3] = [255, 0, 0];
  const BLUE: [u8; 3] = [0, 0, 255];

  fn location(key_color: [u8; 3]) -> Location {
    Location {
      name: format!("{key_color:?}"),
      key_color,
      province: ProvinceId(0),
      terrain: Terrain::Plains,
    }
  }

  /// A raster three cells wide and two tall: red in the west, blue in the east.
  fn raster() -> LocationRaster {
    let pixels = [RED, RED, BLUE, RED, BLUE, BLUE];

    LocationRaster::from_key_colors(3, 2, &pixels, &[location(RED), location(BLUE)]).unwrap()
  }

  #[test]
  fn a_point_lies_in_the_location_painted_under_it() {
    let raster = raster();

    assert_eq!(raster.location_at(MapPosition { x: 0.5, y: 0.5 }), Some(LocationId(0)));
    assert_eq!(raster.location_at(MapPosition { x: 1.5, y: 1.5 }), Some(LocationId(1)));
  }

  #[test]
  fn a_point_past_the_seam_wraps_to_the_other_side() {
    let raster = raster();

    assert_eq!(raster.location_at(MapPosition { x: -0.5, y: 0.5 }), Some(LocationId(1)));
    assert_eq!(raster.location_at(MapPosition { x: 3.5, y: 0.5 }), Some(LocationId(0)));
  }

  #[test]
  fn a_point_north_or_south_of_the_map_is_nowhere() {
    let raster = raster();

    assert_eq!(raster.location_at(MapPosition { x: 0.5, y: -0.1 }), None);
    assert_eq!(raster.location_at(MapPosition { x: 0.5, y: 2.0 }), None);
  }

  #[test]
  fn a_colour_no_location_is_painted_in_is_reported() {
    let pixels = [RED, [1, 2, 3]];

    let result = LocationRaster::from_key_colors(2, 1, &pixels, &[location(RED)]);

    assert_eq!(
      result,
      Err(MapError::UnknownColors {
        colors: vec![[1, 2, 3]]
      })
    );
  }
}

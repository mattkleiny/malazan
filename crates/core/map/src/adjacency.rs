//! Adjacency: which locations share a border.

use std::collections::BTreeSet;

use crate::{locations::LocationId, raster::LocationRaster};

/// The neighbours of every location, found from the location raster.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Adjacency {
  starts: Vec<u32>, // Where each location's neighbours begin, with one entry past the last.
  neighbours: Vec<LocationId>,
}

impl Adjacency {
  /// Finds the locations whose cells touch edge to edge.
  pub fn from_raster(raster: &LocationRaster, location_count: usize) -> Self {
    let mut pairs = BTreeSet::new();

    for row in 0..i64::from(raster.height()) {
      for column in 0..i64::from(raster.width()) {
        let Some(here) = raster.cell(column, row) else {
          continue;
        };

        for there in [raster.cell(column + 1, row), raster.cell(column, row + 1)] {
          if let Some(there) = there
            && there != here
          {
            pairs.insert((here, there));
            pairs.insert((there, here));
          }
        }
      }
    }

    let mut starts = vec![0; location_count + 1];

    for (from, _) in &pairs {
      starts[from.index() + 1] += 1;
    }

    for index in 0..location_count {
      starts[index + 1] += starts[index];
    }

    Self {
      starts,
      neighbours: pairs.into_iter().map(|(_, to)| to).collect(),
    }
  }

  /// The locations bordering `location`, in id order.
  pub fn neighbours_of(&self, location: LocationId) -> &[LocationId] {
    let Some(end) = self.starts.get(location.index() + 1) else {
      return &[];
    };

    &self.neighbours[self.starts[location.index()] as usize..*end as usize]
  }

  pub fn are_neighbours(&self, a: LocationId, b: LocationId) -> bool {
    self.neighbours_of(a).binary_search(&b).is_ok()
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::{
    divisions::ProvinceId,
    locations::{Location, Terrain},
  };

  /// A raster one row tall with one location per cell, west to east.
  fn strip(cells: u16) -> LocationRaster {
    let locations: Vec<Location> = (0..cells)
      .map(|index| Location {
        name: index.to_string(),
        key_color: [index as u8, 0, 0],
        province: ProvinceId(0),
        terrain: Terrain::Plains,
      })
      .collect();
    let pixels: Vec<[u8; 3]> = locations.iter().map(|location| location.key_color).collect();

    LocationRaster::from_key_colors(u32::from(cells), 1, &pixels, &locations).unwrap()
  }

  #[test]
  fn locations_that_touch_are_neighbours() {
    let adjacency = Adjacency::from_raster(&strip(4), 4);

    assert_eq!(adjacency.neighbours_of(LocationId(1)), [LocationId(0), LocationId(2)]);
    assert!(!adjacency.are_neighbours(LocationId(0), LocationId(2)));
  }

  #[test]
  fn locations_at_opposite_edges_are_not_neighbours() {
    let adjacency = Adjacency::from_raster(&strip(4), 4);

    assert!(!adjacency.are_neighbours(LocationId(0), LocationId(3)));
  }

  #[test]
  fn a_location_is_never_its_own_neighbour() {
    let adjacency = Adjacency::from_raster(&strip(1), 1);

    assert!(adjacency.neighbours_of(LocationId(0)).is_empty());
  }
}

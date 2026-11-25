//! Writing a map back out to the files it is read from.

use crate::{
  images::{encode_grey16, encode_rgb8},
  map::{Map, MapError},
};

/// The contents of a map's three source files.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MapFiles {
  pub definitions: String,
  pub locations_png: Vec<u8>,
  pub heightmap_png: Vec<u8>,
}

impl Map {
  /// The map as its source files would hold it.
  pub fn to_files(&self) -> Result<MapFiles, MapError> {
    let painting: Vec<u8> = self
      .raster
      .cells()
      .iter()
      .flat_map(|cell| {
        self
          .locations
          .get(cell.index())
          .map_or([0; 3], |location| location.key_color)
      })
      .collect();

    Ok(MapFiles {
      definitions: self.definitions_text(),
      locations_png: encode_rgb8(self.raster.width(), self.raster.height(), &painting)
        .ok_or(MapError::UnwritableImage { image: "locations" })?,
      heightmap_png: encode_grey16(self.heights.width(), self.heights.height(), self.heights.samples())
        .ok_or(MapError::UnwritableImage { image: "heightmap" })?,
    })
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::{
    heights::Heightfield,
    map::{MapParts, MapSources},
  };

  #[test]
  fn a_written_map_reads_back_the_same() {
    let map = Map::from_parts(MapParts {
      definitions: "1,0,0,Land,P,A,R,C,plains\n2,0,0,Bay,Q,B,S,D,sea",
      width: 3,
      height: 2,
      key_colors: &[[1, 0, 0], [2, 0, 0], [2, 0, 0], [1, 0, 0], [1, 0, 0], [2, 0, 0]],
      heights: Heightfield::from_samples(2, 1, vec![100, 60_000]).unwrap(),
      sea_level: 0.3,
    })
    .unwrap();

    let files = map.to_files().unwrap();
    let read = Map::from_sources(&MapSources {
      definitions: &files.definitions,
      locations_png: &files.locations_png,
      heightmap_png: &files.heightmap_png,
      sea_level: 0.3,
    })
    .unwrap();

    assert_eq!(read.raster, map.raster);
    assert_eq!(read.heights, map.heights);
    assert_eq!(read.locations, map.locations);
    assert_eq!(read.provinces, map.provinces);
  }
}

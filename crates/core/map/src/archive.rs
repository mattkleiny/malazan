//! The map file: one zip archive holding the definitions, the location image,
//! the heightmap and the map labels.

use std::io::{Cursor, Read, Write};

use zip::{CompressionMethod, ZipArchive, ZipWriter, write::SimpleFileOptions};

use crate::{
  labels::{MapLabel, labels_text, parse_labels},
  map::{Map, MapError, MapSources},
};

const DEFINITIONS: &str = "definitions.csv";
const LOCATIONS: &str = "locations.png";
const HEIGHTMAP: &str = "heightmap.png";
const LABELS: &str = "labels.csv";

/// What a map file holds: the map, and the labels lettered across it.
#[derive(Clone, Debug)]
pub struct MapContents {
  pub map: Map,
  pub labels: Vec<MapLabel>,
}

impl MapContents {
  /// Reads the bytes of a map file; a file without labels has none.
  pub fn from_archive(bytes: &[u8], sea_level: f64) -> Result<Self, MapError> {
    let mut archive = ZipArchive::new(Cursor::new(bytes)).map_err(|_| MapError::UnreadableArchive)?;

    let definitions = read_text(&mut archive, DEFINITIONS)?;
    let locations = read_entry(&mut archive, LOCATIONS)?;
    let heightmap = read_entry(&mut archive, HEIGHTMAP)?;
    let labels = match archive.index_for_name(LABELS) {
      Some(_) => parse_labels(&read_text(&mut archive, LABELS)?).map_err(MapError::MalformedLabels)?,
      None => Vec::new(),
    };

    let map = Map::from_sources(&MapSources {
      definitions: &definitions,
      locations_png: &locations,
      heightmap_png: &heightmap,
      sea_level,
    })?;

    Ok(Self { map, labels })
  }

  /// The bytes of the map file holding these contents.
  pub fn to_archive(&self) -> Result<Vec<u8>, MapError> {
    let files = self.map.to_files()?;
    let labels = labels_text(&self.labels);
    // The images are compressed already; only the tables gain from it.
    let entries = [
      (DEFINITIONS, files.definitions.as_bytes(), CompressionMethod::Deflated),
      (LOCATIONS, files.locations_png.as_slice(), CompressionMethod::Stored),
      (HEIGHTMAP, files.heightmap_png.as_slice(), CompressionMethod::Stored),
      (LABELS, labels.as_bytes(), CompressionMethod::Deflated),
    ];

    let mut writer = ZipWriter::new(Cursor::new(Vec::new()));

    for (name, bytes, compression) in entries {
      let options = SimpleFileOptions::default().compression_method(compression);

      writer
        .start_file(name, options)
        .map_err(|_| MapError::UnwritableArchive)?;

      writer.write_all(bytes).map_err(|_| MapError::UnwritableArchive)?;
    }

    let written = writer.finish().map_err(|_| MapError::UnwritableArchive)?;

    Ok(written.into_inner())
  }
}

fn read_entry(archive: &mut ZipArchive<Cursor<&[u8]>>, entry: &'static str) -> Result<Vec<u8>, MapError> {
  let mut file = archive.by_name(entry).map_err(|_| MapError::MissingEntry { entry })?;
  let mut bytes = Vec::new();

  file
    .read_to_end(&mut bytes)
    .map_err(|_| MapError::UnreadableEntry { entry })?;

  Ok(bytes)
}

fn read_text(archive: &mut ZipArchive<Cursor<&[u8]>>, entry: &'static str) -> Result<String, MapError> {
  String::from_utf8(read_entry(archive, entry)?).map_err(|_| MapError::UnreadableEntry { entry })
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::{
    heights::Heightfield,
    map::{MapParts, MapPosition},
  };

  fn contents() -> MapContents {
    let map = Map::from_parts(MapParts {
      definitions: "1,0,0,Land,P,A,R,C,plains\n2,0,0,Bay,Q,B,S,D,sea",
      width: 3,
      height: 2,
      key_colors: &[[1, 0, 0], [2, 0, 0], [2, 0, 0], [1, 0, 0], [1, 0, 0], [2, 0, 0]],
      heights: Heightfield::from_samples(2, 1, vec![100, 60_000]).unwrap(),
      sea_level: 0.3,
    })
    .unwrap();
    let labels = vec![MapLabel {
      text: "The Bay".to_string(),
      position: MapPosition { x: 2.5, y: 0.5 },
      size: 12.0,
      angle: 0.0,
      is_water: true,
    }];

    MapContents { map, labels }
  }

  /// A map file holding only the given entries.
  fn archive_with(entries: &[(&str, &[u8])]) -> Vec<u8> {
    let mut writer = ZipWriter::new(Cursor::new(Vec::new()));
    for (name, bytes) in entries {
      writer.start_file(*name, SimpleFileOptions::default()).unwrap();
      writer.write_all(bytes).unwrap();
    }
    writer.finish().unwrap().into_inner()
  }

  #[test]
  fn a_written_map_file_reads_back_the_same() {
    let contents = contents();

    let read = MapContents::from_archive(&contents.to_archive().unwrap(), 0.3).unwrap();

    assert_eq!(read.map.raster, contents.map.raster);
    assert_eq!(read.map.heights, contents.map.heights);
    assert_eq!(read.map.locations, contents.map.locations);
    assert_eq!(read.map.provinces, contents.map.provinces);
    assert_eq!(read.labels, contents.labels);
  }

  #[test]
  fn a_map_file_without_labels_has_none() {
    let contents = contents();
    let files = contents.map.to_files().unwrap();
    let archive = archive_with(&[
      (DEFINITIONS, files.definitions.as_bytes()),
      (LOCATIONS, &files.locations_png),
      (HEIGHTMAP, &files.heightmap_png),
    ]);

    let read = MapContents::from_archive(&archive, 0.3).unwrap();

    assert!(read.labels.is_empty());
  }

  #[test]
  fn a_map_file_that_is_not_an_archive_is_reported() {
    let result = MapContents::from_archive(b"nope", 0.3);

    assert_eq!(result.err(), Some(MapError::UnreadableArchive));
  }

  #[test]
  fn a_map_file_missing_its_heightmap_is_reported() {
    let archive = archive_with(&[(DEFINITIONS, b""), (LOCATIONS, b"")]);

    let result = MapContents::from_archive(&archive, 0.3);

    assert_eq!(
      result.err().map(|error| error.to_string()),
      Some("the map file holds no heightmap.png".to_string())
    );
  }
}

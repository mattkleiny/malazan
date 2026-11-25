//! The map as a whole: its locations, the tiers above them, and its shape.

use std::fmt;

use surreal::{cvar, ecs::Component};

use crate::{
  adjacency::Adjacency,
  definitions::parse_definitions,
  divisions::{Area, Continent, Province, Region},
  heights::Heightfield,
  images::decode_png,
  locations::{Location, LocationId},
  raster::LocationRaster,
};

cvar!(pub SEA_LEVEL: f64, "map/sea_level", 0.3, "The height (0..1) of the heightmap at which the sea lies.");

/// A point on the map (map units): `x` runs east and wraps at the seam, `y`
/// runs south from the northern edge.
#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub struct MapPosition {
  pub x: f64,
  pub y: f64,
}

/// Why a map could not be read from its sources.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MapError {
  MalformedRow { line: usize },
  UnknownTerrain { line: usize, terrain: String },
  DuplicateColor { line: usize, color: [u8; 3] },
  DuplicateLocation { line: usize, name: String },
  SplitDivision { line: usize, name: String },
  TooManyLocations,
  NoLocations,
  UnreadableImage { image: &'static str },
  UnknownColors { colors: Vec<[u8; 3]> },
}

impl fmt::Display for MapError {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Self::MalformedRow { line } => write!(f, "definitions line {line}: expected nine filled columns"),
      Self::UnknownTerrain { line, terrain } => write!(f, "definitions line {line}: unknown terrain '{terrain}'"),
      Self::DuplicateColor { line, color } => {
        write!(f, "definitions line {line}: key colour {color:?} is already taken")
      }
      Self::DuplicateLocation { line, name } => {
        write!(f, "definitions line {line}: location '{name}' is already defined")
      }
      Self::SplitDivision { line, name } => {
        write!(
          f,
          "definitions line {line}: '{name}' is placed under two different parents"
        )
      }
      Self::TooManyLocations => write!(f, "the map defines more locations or divisions than it can hold"),
      Self::NoLocations => write!(f, "the definitions name no locations"),
      Self::UnreadableImage { image } => write!(f, "the {image} image could not be read"),
      Self::UnknownColors { colors } => write!(f, "the locations image uses undefined key colours: {colors:?}"),
    }
  }
}

/// The files a map is read from.
pub struct MapSources<'a> {
  pub definitions: &'a str,    // The definitions table.
  pub locations_png: &'a [u8], // The image painted in key colours.
  pub heightmap_png: &'a [u8], // The greyscale heightmap.
  pub sea_level: f64,          // Height of the sea on the heightmap (0..1).
}

/// A map's contents once its files are decoded.
pub struct MapParts<'a> {
  pub definitions: &'a str,      // The definitions table.
  pub width: u32,                // Width of the painting (map units).
  pub height: u32,               // Height of the painting (map units).
  pub key_colors: &'a [[u8; 3]], // The painting, row by row from the north-west corner.
  pub heights: Heightfield,
  pub sea_level: f64, // Height of the sea on the heightfield (0..1).
}

/// The world's breakdown into locations, and the lie of its land.
#[derive(Component, Clone, Debug)]
pub struct Map {
  pub locations: Vec<Location>,
  pub provinces: Vec<Province>,
  pub areas: Vec<Area>,
  pub regions: Vec<Region>,
  pub continents: Vec<Continent>,
  pub raster: LocationRaster,
  pub heights: Heightfield,
  pub adjacency: Adjacency,
  pub sea_level: f64, // Height of the sea on the heightfield (0..1).
}

impl Map {
  /// Reads a map from its source files.
  pub fn from_sources(sources: &MapSources) -> Result<Self, MapError> {
    let painted = decode_png(sources.locations_png).ok_or(MapError::UnreadableImage { image: "locations" })?;
    let relief = decode_png(sources.heightmap_png).ok_or(MapError::UnreadableImage { image: "heightmap" })?;

    let heights = Heightfield::from_samples(relief.width, relief.height, relief.to_grey16())
      .ok_or(MapError::UnreadableImage { image: "heightmap" })?;

    Self::from_parts(MapParts {
      definitions: sources.definitions,
      width: painted.width,
      height: painted.height,
      key_colors: &painted.to_rgb8(),
      heights,
      sea_level: sources.sea_level,
    })
  }

  /// Puts a map together from its definitions, painting and heights.
  pub fn from_parts(parts: MapParts) -> Result<Self, MapError> {
    let definitions = parse_definitions(parts.definitions)?;
    let raster = LocationRaster::from_key_colors(parts.width, parts.height, parts.key_colors, &definitions.locations)?;
    let adjacency = Adjacency::from_raster(&raster, definitions.locations.len());

    Ok(Self {
      locations: definitions.locations,
      provinces: definitions.provinces,
      areas: definitions.areas,
      regions: definitions.regions,
      continents: definitions.continents,
      raster,
      heights: parts.heights,
      adjacency,
      sea_level: parts.sea_level,
    })
  }

  /// Distance from the seam eastwards and back to it (map units).
  pub fn width(&self) -> f64 {
    f64::from(self.raster.width())
  }

  /// Distance from the northern edge to the southern (map units).
  pub fn height(&self) -> f64 {
    f64::from(self.raster.height())
  }

  pub fn location(&self, id: LocationId) -> Option<&Location> {
    self.locations.get(id.index())
  }

  pub fn province_of(&self, id: LocationId) -> Option<&Province> {
    self.provinces.get(usize::from(self.location(id)?.province.0))
  }

  pub fn area_of(&self, id: LocationId) -> Option<&Area> {
    self.areas.get(usize::from(self.province_of(id)?.area.0))
  }

  pub fn region_of(&self, id: LocationId) -> Option<&Region> {
    self.regions.get(usize::from(self.area_of(id)?.region.0))
  }

  pub fn continent_of(&self, id: LocationId) -> Option<&Continent> {
    self.continents.get(usize::from(self.region_of(id)?.continent.0))
  }

  /// The location under a point of the map.
  pub fn location_at(&self, position: MapPosition) -> Option<LocationId> {
    self.raster.location_at(position)
  }

  /// How far the ground at a point stands above the sea, as a share of the
  /// heightfield's full range; negative under water.
  pub fn elevation_at(&self, position: MapPosition) -> f64 {
    self
      .heights
      .height_at(position.x / self.width(), position.y / self.height())
      - self.sea_level
  }

  /// The same point, brought back between the seam and its far side.
  pub fn wrapped(&self, position: MapPosition) -> MapPosition {
    MapPosition {
      x: position.x.rem_euclid(self.width()),
      y: position.y,
    }
  }
}

#[cfg(test)]
mod tests {
  use png::{BitDepth, ColorType, Encoder};

  use super::*;
  use crate::locations::Terrain;

  const DEFINITIONS: &str = "\
255,0,0,Aren,Aren Coast,Raraku,Seven Cities,Quon Tali,desert
0,0,255,Aren Bay,Aren Waters,Raraku,Seven Cities,Quon Tali,sea
";

  fn png(width: u32, height: u32, color: ColorType, depth: BitDepth, data: &[u8]) -> Vec<u8> {
    let mut bytes = Vec::new();
    let mut encoder = Encoder::new(&mut bytes, width, height);
    encoder.set_color(color);
    encoder.set_depth(depth);
    encoder.write_header().unwrap().write_image_data(data).unwrap();
    bytes
  }

  /// A map four units wide and one tall: land in the west half, sea in the
  /// east.
  fn map() -> Map {
    let locations = png(4, 1, ColorType::Rgb, BitDepth::Eight, &[
      255, 0, 0, 255, 0, 0, 0, 0, 255, 0, 0, 255,
    ]);
    let heightmap = png(2, 1, ColorType::Grayscale, BitDepth::Eight, &[255, 0]);

    Map::from_sources(&MapSources {
      definitions: DEFINITIONS,
      locations_png: &locations,
      heightmap_png: &heightmap,
      sea_level: 0.5,
    })
    .unwrap()
  }

  #[test]
  fn a_map_knows_what_lies_under_a_point() {
    let map = map();

    let location = map.location_at(MapPosition { x: 2.5, y: 0.5 }).unwrap();

    assert_eq!(map.location(location).unwrap().name, "Aren Bay");
    assert_eq!(map.location(location).unwrap().terrain, Terrain::Sea);
  }

  #[test]
  fn a_location_sits_inside_every_tier_above_it() {
    let map = map();

    assert_eq!(map.province_of(LocationId(0)).unwrap().name, "Aren Coast");
    assert_eq!(map.area_of(LocationId(0)).unwrap().name, "Raraku");
    assert_eq!(map.region_of(LocationId(0)).unwrap().name, "Seven Cities");
    assert_eq!(map.continent_of(LocationId(0)).unwrap().name, "Quon Tali");
  }

  #[test]
  fn land_stands_above_the_sea_and_the_seabed_below_it() {
    let map = map();

    assert!(map.elevation_at(MapPosition { x: 1.0, y: 0.5 }) > 0.0);
    assert!(map.elevation_at(MapPosition { x: 3.0, y: 0.5 }) < 0.0);
  }

  #[test]
  fn the_map_takes_its_size_from_the_location_image() {
    let map = map();

    assert_eq!((map.width(), map.height()), (4.0, 1.0));
    assert!(map.adjacency.are_neighbours(LocationId(0), LocationId(1)));
  }

  #[test]
  fn a_point_past_the_seam_wraps_back_onto_the_map() {
    let map = map();

    assert_eq!(map.wrapped(MapPosition { x: -1.0, y: 0.5 }), MapPosition {
      x: 3.0,
      y: 0.5
    });
  }

  #[test]
  fn a_heightmap_that_is_not_an_image_is_reported() {
    let locations = png(1, 1, ColorType::Rgb, BitDepth::Eight, &[255, 0, 0]);

    let result = Map::from_sources(&MapSources {
      definitions: DEFINITIONS,
      locations_png: &locations,
      heightmap_png: b"nope",
      sea_level: 0.5,
    });

    assert_eq!(result.err(), Some(MapError::UnreadableImage { image: "heightmap" }));
  }
}

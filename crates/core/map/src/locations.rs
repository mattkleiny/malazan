//! Locations: the smallest pieces the map is divided into.

use crate::divisions::ProvinceId;

/// Names a location by its place in the map's table of them.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct LocationId(pub u16);

impl LocationId {
  /// The location's row in tables indexed by location.
  pub fn index(self) -> usize {
    usize::from(self.0)
  }
}

/// The kind of ground or water a location is made of.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum Terrain {
  Plains,
  Forest,
  Hills,
  Mountains,
  Desert,
  Marsh,
  Tundra,
  Sea,
  Lake,
}

impl Terrain {
  pub const ALL: [Terrain; 9] = [
    Terrain::Plains,
    Terrain::Forest,
    Terrain::Hills,
    Terrain::Mountains,
    Terrain::Desert,
    Terrain::Marsh,
    Terrain::Tundra,
    Terrain::Sea,
    Terrain::Lake,
  ];

  /// The name map definitions use for this terrain.
  pub fn key(self) -> &'static str {
    match self {
      Terrain::Plains => "plains",
      Terrain::Forest => "forest",
      Terrain::Hills => "hills",
      Terrain::Mountains => "mountains",
      Terrain::Desert => "desert",
      Terrain::Marsh => "marsh",
      Terrain::Tundra => "tundra",
      Terrain::Sea => "sea",
      Terrain::Lake => "lake",
    }
  }

  /// The terrain a map definition names.
  pub fn from_key(key: &str) -> Option<Terrain> {
    Terrain::ALL.into_iter().find(|terrain| terrain.key() == key)
  }

  pub fn is_water(self) -> bool {
    matches!(self, Terrain::Sea | Terrain::Lake)
  }
}

/// One location of the map.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Location {
  pub name: String,
  pub key_color: [u8; 3], // The colour it is painted in on the location image.
  pub province: ProvinceId,
  pub terrain: Terrain,
}

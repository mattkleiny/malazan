//! The tiers above locations: provinces, areas, regions and continents.

/// Names a province by its place in the map's table of them.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ProvinceId(pub u16);

/// Names an area by its place in the map's table of them.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AreaId(pub u16);

/// Names a region by its place in the map's table of them.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RegionId(pub u16);

/// Names a continent by its place in the map's table of them.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ContinentId(pub u16);

/// A group of neighbouring locations.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Province {
  pub name: String,
  pub area: AreaId,
}

/// A group of provinces.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Area {
  pub name: String,
  pub region: RegionId,
}

/// A group of areas.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Region {
  pub name: String,
  pub continent: ContinentId,
}

/// The largest division of the map.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Continent {
  pub name: String,
}

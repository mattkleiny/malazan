//! The map: the world's breakdown into locations and the tiers above them.

mod adjacency;
mod definitions;
mod divisions;
mod heights;
mod images;
mod locations;
mod map;
mod modes;
mod raster;

pub use adjacency::Adjacency;
pub use divisions::{Area, AreaId, Continent, ContinentId, Province, ProvinceId, Region, RegionId};
pub use heights::Heightfield;
pub use locations::{Location, LocationId, Terrain};
pub use map::{Map, MapError, MapParts, MapPosition, MapSources, SEA_LEVEL};
pub use modes::{
  Fill, MapMode, MapModeRegistration, map_mode, map_modes,
  standard::{division_color, land_fills},
};
pub use raster::LocationRaster;

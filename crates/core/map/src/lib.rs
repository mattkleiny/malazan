//! The map: the world's breakdown into locations and the tiers above them.

mod adjacency;
mod definitions;
mod divisions;
mod editing;
mod heights;
mod images;
mod locations;
mod map;
mod modes;
mod raster;
mod selections;

pub use adjacency::Adjacency;
pub use divisions::{Area, AreaId, Continent, ContinentId, Province, ProvinceId, Region, RegionId};
pub use editing::{
  CellRect,
  brushes::{Brush, Sculpt, paint_location, sculpt},
  definitions::LocationField,
  files::MapFiles,
  patches::{MapEdit, MapPatch, Stroke},
};
pub use heights::Heightfield;
pub use locations::{Location, LocationId, Terrain};
pub use map::{Map, MapError, MapParts, MapPosition, MapSources, SEA_LEVEL};
pub use modes::{
  Fill, MapMode, MapModeRegistration, map_mode, map_modes,
  standard::{division_color, land_fills},
};
pub use raster::LocationRaster;
pub use selections::{Selection, Tier, division_selection, location_selection};

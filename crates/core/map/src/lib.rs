//! The map: the world's breakdown into locations and the tiers above them.

mod adjacency;
mod archive;
mod definitions;
mod divisions;
mod editing;
mod heights;
mod images;
mod labels;
mod locations;
mod map;
mod modes;
mod raster;
mod resource;
mod selections;

pub use archive::MapContents;
pub use editing::{
  CellRect,
  brushes::{Brush, Sculpt, paint_location, sculpt},
  definitions::LocationField,
  patches::{MapEdit, Stroke},
};
pub use heights::Heightfield;
pub use labels::MapLabel;
pub use locations::{Location, LocationId, Terrain};
pub use map::{Map, MapParts, MapPosition};
pub use modes::{Fill, MapMode, map_mode, map_modes};
pub use resource::MapResource;
pub use selections::{Selection, Tier, division_selection};

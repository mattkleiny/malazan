//! Terrain: drawing the map's surface and finding what lies under the cursor.

mod borders;
mod chunks;
mod picking;
mod transitions;
mod view;

pub use borders::{BorderRank, CellBorders, cell_borders};
pub use chunks::{ChunkGrid, ChunkMesh, LEVEL_QUADS, chunk_mesh, level_of_detail, nearest_copy};
pub use picking::{Pick, Ray, pick, surface_height};
pub use transitions::FillTransition;
pub use view::{MAP_MODE_FADE, TerrainView};

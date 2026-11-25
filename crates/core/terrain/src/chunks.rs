//! Chunks: the pieces the terrain's surface is drawn in.

/// The size chunks are cut to when the map divides evenly (map units).
const TARGET_CHUNK_SIZE: f64 = 128.0;

/// Quads along a chunk's side at each level of detail, finest first.
pub const LEVEL_QUADS: [u32; 4] = [64, 32, 16, 8];

/// How the map is cut into chunks of equal size.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct ChunkGrid {
  pub columns: u32,
  pub rows: u32,
  pub chunk_width: f64, // (map units)
  pub chunk_depth: f64, // (map units)
}

impl ChunkGrid {
  /// Cuts a map of the given size into chunks that cover it exactly.
  pub fn covering(map_width: f64, map_height: f64) -> Self {
    let columns = (map_width / TARGET_CHUNK_SIZE).round().max(1.0);
    let rows = (map_height / TARGET_CHUNK_SIZE).round().max(1.0);

    Self {
      columns: columns as u32,
      rows: rows as u32,
      chunk_width: map_width / columns,
      chunk_depth: map_height / rows,
    }
  }

  /// The north-west corner of a chunk (map units).
  pub fn origin(&self, column: u32, row: u32) -> [f64; 2] {
    [f64::from(column) * self.chunk_width, f64::from(row) * self.chunk_depth]
  }
}

/// The flat mesh of one chunk: a grid with its north-west corner at the
/// origin, and a skirt hanging from its rim to hide gaps between levels of
/// detail. Skirt vertices sit at `y = -1`, the rest at `y = 0`.
#[derive(Clone, Debug, PartialEq)]
pub struct ChunkMesh {
  pub vertices: Vec<[f32; 3]>,
  pub indices: Vec<u32>,
}

/// Builds the mesh of a chunk `width` by `depth`, `quads` to a side.
pub fn chunk_mesh(width: f64, depth: f64, quads: u32) -> ChunkMesh {
  let side = quads + 1;
  let at = |column: u32, row: u32| row * side + column;

  let mut vertices = Vec::with_capacity((side * side + side * 4) as usize);
  let mut indices = Vec::with_capacity((quads * quads * 6 + quads * 4 * 12) as usize);

  for row in 0..side {
    for column in 0..side {
      let x = width * f64::from(column) / f64::from(quads);
      let z = depth * f64::from(row) / f64::from(quads);

      vertices.push([x as f32, 0.0, z as f32]);
    }
  }

  // Front faces wind clockwise seen from above.
  for row in 0..quads {
    for column in 0..quads {
      let (north_west, north_east) = (at(column, row), at(column + 1, row));
      let (south_west, south_east) = (at(column, row + 1), at(column + 1, row + 1));

      indices.extend([north_west, north_east, south_west, north_east, south_east, south_west]);
    }
  }

  let rims: [Vec<u32>; 4] = [
    (0..side).map(|column| at(column, 0)).collect(),
    (0..side).map(|column| at(column, quads)).collect(),
    (0..side).map(|row| at(0, row)).collect(),
    (0..side).map(|row| at(quads, row)).collect(),
  ];

  for rim in rims {
    let hem = vertices.len() as u32;

    for &vertex in &rim {
      let [x, _, z] = vertices[vertex as usize];

      vertices.push([x, -1.0, z]);
    }

    // Both windings, so a skirt shows from either side.
    for step in 0..quads {
      let (top, next_top) = (rim[step as usize], rim[step as usize + 1]);
      let (bottom, next_bottom) = (hem + step, hem + step + 1);

      indices.extend([top, next_top, bottom, next_top, next_bottom, bottom]);
      indices.extend([top, bottom, next_top, next_top, bottom, next_bottom]);
    }
  }

  ChunkMesh { vertices, indices }
}

/// The copy of `x` nearest to `focus` on a map that repeats every `width`.
pub fn nearest_copy(x: f64, focus: f64, width: f64) -> f64 {
  x + ((focus - x) / width).round() * width
}

/// The level of detail for a chunk at `distance` from the camera, where the
/// finest level reaches out to `reach` and each coarser one twice as far.
pub fn level_of_detail(distance: f64, reach: f64) -> usize {
  let coarsest = LEVEL_QUADS.len() - 1;

  if distance <= reach || reach <= 0.0 {
    return 0;
  }

  (((distance / reach).log2().floor() as usize) + 1).min(coarsest)
}

#[cfg(test)]
mod tests {
  use super::*;

  /// The way a triangle faces, as the engine sees it from its winding.
  fn facing(mesh: &ChunkMesh, triangle: &[u32]) -> [f32; 3] {
    let [a, b, c] = [0, 1, 2].map(|corner| mesh.vertices[triangle[corner] as usize]);
    let (u, v) = ([c[0] - a[0], c[1] - a[1], c[2] - a[2]], [
      b[0] - a[0],
      b[1] - a[1],
      b[2] - a[2],
    ]);

    [
      u[1] * v[2] - u[2] * v[1],
      u[2] * v[0] - u[0] * v[2],
      u[0] * v[1] - u[1] * v[0],
    ]
  }

  #[test]
  fn chunks_cover_the_map_exactly() {
    let grid = ChunkGrid::covering(2048.0, 1000.0);

    assert_eq!((grid.columns, grid.rows), (16, 8));
    assert_eq!(f64::from(grid.columns) * grid.chunk_width, 2048.0);
    assert_eq!(f64::from(grid.rows) * grid.chunk_depth, 1000.0);
    assert_eq!(grid.origin(1, 2), [128.0, 250.0]);
  }

  #[test]
  fn a_map_smaller_than_a_chunk_is_one_chunk() {
    let grid = ChunkGrid::covering(40.0, 20.0);

    assert_eq!((grid.columns, grid.rows), (1, 1));
    assert_eq!((grid.chunk_width, grid.chunk_depth), (40.0, 20.0));
  }

  #[test]
  fn a_chunk_s_surface_spans_its_size_and_faces_up() {
    let mesh = chunk_mesh(128.0, 64.0, 4);

    let surface = &mesh.indices[..4 * 4 * 6];

    assert!(surface.chunks_exact(3).all(|triangle| facing(&mesh, triangle)[1] > 0.0));
    assert!(mesh.vertices.contains(&[128.0, 0.0, 64.0]));
    assert!(
      mesh
        .vertices
        .iter()
        .all(|[x, _, z]| (0.0..=128.0).contains(x) && (0.0..=64.0).contains(z))
    );
  }

  #[test]
  fn a_skirt_hangs_from_every_rim_vertex() {
    let mesh = chunk_mesh(8.0, 8.0, 4);

    let hem = mesh.vertices.iter().filter(|vertex| vertex[1] == -1.0).count();

    assert_eq!(hem, 5 * 4);
    assert_eq!(mesh.vertices.len(), 5 * 5 + hem);
    assert!(mesh.indices.iter().all(|&index| (index as usize) < mesh.vertices.len()));
  }

  #[test]
  fn a_chunk_is_drawn_at_its_copy_nearest_the_camera() {
    assert_eq!(nearest_copy(10.0, 20.0, 100.0), 10.0);
    assert_eq!(nearest_copy(10.0, 90.0, 100.0), 110.0);
    assert_eq!(nearest_copy(90.0, -5.0, 100.0), -10.0);
    assert_eq!(nearest_copy(10.0, 512.0, 100.0), 510.0);
  }

  #[test]
  fn detail_falls_off_with_distance() {
    assert_eq!(level_of_detail(0.0, 100.0), 0);
    assert_eq!(level_of_detail(100.0, 100.0), 0);
    assert_eq!(level_of_detail(150.0, 100.0), 1);
    assert_eq!(level_of_detail(250.0, 100.0), 2);
    assert_eq!(level_of_detail(1e9, 100.0), LEVEL_QUADS.len() - 1);
  }
}

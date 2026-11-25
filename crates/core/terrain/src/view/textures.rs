//! The textures the terrain shader reads the map from.

use game_map::{CellRect, Fill, Map};
use godot::{
  classes::{Image, ImageTexture, image::Format},
  prelude::*,
};

use crate::borders::{CellBorders, borders_of, cell_borders};

/// Width of the colour lookup texture; locations fill it row by row.
pub const LOOKUP_WIDTH: usize = 256;

/// One texture of the map, with a copy of its texels kept to patch.
pub struct Layer {
  pub texture: Gd<ImageTexture>,
  texels: Vec<u8>,
  width: u32,
  height: u32,
  format: Format,
  texel_size: usize, // Bytes per texel.
}

impl Layer {
  fn new(width: u32, height: u32, format: Format, texel_size: usize, texels: Vec<u8>) -> Option<Layer> {
    let image = image(width, height, format, &texels)?;

    Some(Layer {
      texture: ImageTexture::create_from_image(&image)?,
      texels,
      width,
      height,
      format,
      texel_size,
    })
  }

  /// Rewrites a block of texels from `texel_at`, then sends the texture to the
  /// GPU again.
  fn patch(&mut self, rect: CellRect, texel_at: impl Fn(u32, u32) -> Vec<u8>) {
    for (column, row) in rect.cells() {
      if column >= self.width || row >= self.height {
        continue;
      }

      let start = (row as usize * self.width as usize + column as usize) * self.texel_size;
      self.texels[start..start + self.texel_size].copy_from_slice(&texel_at(column, row));
    }

    if let Some(image) = image(self.width, self.height, self.format, &self.texels) {
      self.texture.update(&image);
    }
  }

  fn whole(&self) -> CellRect {
    CellRect {
      column: 0,
      row: 0,
      width: self.width,
      height: self.height,
    }
  }
}

/// Every texture the shader needs to draw one map.
pub struct MapTextures {
  pub heights: Layer,
  pub locations: Layer,
  pub borders: Layer,
}

impl MapTextures {
  pub fn from_map(map: &Map) -> Option<Self> {
    let heights = map
      .heights
      .samples()
      .iter()
      .flat_map(|&sample| height_texel(sample))
      .collect();
    let locations = map
      .raster
      .cells()
      .iter()
      .flat_map(|location| location.0.to_le_bytes())
      .collect();
    let borders = cell_borders(map).into_iter().flat_map(border_texel).collect();

    Some(Self {
      heights: Layer::new(map.heights.width(), map.heights.height(), Format::RF, 4, heights)?,
      locations: Layer::new(map.raster.width(), map.raster.height(), Format::RG8, 2, locations)?,
      borders: Layer::new(map.raster.width(), map.raster.height(), Format::RG8, 2, borders)?,
    })
  }

  /// Redraws a block of the painting and the borders around it.
  pub fn refresh_cells(&mut self, map: &Map, rect: CellRect) {
    self.locations.patch(rect, |column, row| {
      let cell = map
        .raster
        .cell(i64::from(column), i64::from(row))
        .map_or(0, |location| location.0);
      cell.to_le_bytes().to_vec()
    });

    // A cell's borders look east and south, so the cells west and north of the
    // block change too.
    self.borders.patch(rect.grown_north_west(1), |column, row| {
      border_texel(borders_of(map, i64::from(column), i64::from(row))).to_vec()
    });
  }

  /// Redraws every border, after the divisions have changed.
  pub fn refresh_borders(&mut self, map: &Map) {
    let whole = self.borders.whole();

    self.borders.patch(whole, |column, row| {
      border_texel(borders_of(map, i64::from(column), i64::from(row))).to_vec()
    });
  }

  /// Redraws a block of the heightfield.
  pub fn refresh_heights(&mut self, map: &Map, rect: CellRect) {
    let width = map.heights.width() as usize;

    self.heights.patch(rect, |column, row| {
      height_texel(map.heights.samples()[row as usize * width + column as usize]).to_vec()
    });
  }
}

fn height_texel(sample: u16) -> [u8; 4] {
  (f32::from(sample) / f32::from(u16::MAX)).to_le_bytes()
}

fn border_texel(borders: CellBorders) -> [u8; 2] {
  [borders.east.to_texel(), borders.south.to_texel()]
}

fn image(width: u32, height: u32, format: Format, texels: &[u8]) -> Option<Gd<Image>> {
  let data = PackedByteArray::from(texels);

  Image::create_from_data(width as i32, height as i32, false, format, &data)
}

/// The colour lookup the shader reads fills from: one texel per location,
/// row by row.
pub fn colors_image(fills: &[Fill]) -> Option<Gd<Image>> {
  let rows = fills.len().div_ceil(LOOKUP_WIDTH).max(1);
  let texels = fills
    .iter()
    .copied()
    .chain(std::iter::repeat(Fill::CLEAR))
    .take(rows * LOOKUP_WIDTH);
  let data: PackedByteArray = texels
    .flat_map(|fill| [fill.color[0], fill.color[1], fill.color[2], fill.strength])
    .collect();

  Image::create_from_data(LOOKUP_WIDTH as i32, rows as i32, false, Format::RGBA8, &data)
}

/// Which locations are lit as hovered (red) and selected (green), laid out
/// like the colour lookup.
pub fn highlights_image(lit: &[[bool; 2]]) -> Option<Gd<Image>> {
  let rows = lit.len().div_ceil(LOOKUP_WIDTH).max(1);
  let texels = lit
    .iter()
    .copied()
    .chain(std::iter::repeat([false; 2]))
    .take(rows * LOOKUP_WIDTH);
  let data: PackedByteArray = texels
    .flat_map(|[hovered, selected]| [u8::from(hovered) * u8::MAX, u8::from(selected) * u8::MAX])
    .collect();

  Image::create_from_data(LOOKUP_WIDTH as i32, rows as i32, false, Format::RG8, &data)
}

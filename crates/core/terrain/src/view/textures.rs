//! The textures the terrain shader reads the map from.

use game_map::{Fill, Map};
use godot::{
  classes::{Image, ImageTexture, image::Format},
  prelude::*,
};

use crate::borders::cell_borders;

/// Width of the colour lookup texture; locations fill it row by row.
pub const LOOKUP_WIDTH: usize = 256;

/// Every texture the shader needs to draw one map.
pub struct MapTextures {
  pub heights: Gd<ImageTexture>,
  pub locations: Gd<ImageTexture>,
  pub borders: Gd<ImageTexture>,
}

impl MapTextures {
  pub fn from_map(map: &Map) -> Option<Self> {
    let heights = map
      .heights
      .samples()
      .iter()
      .flat_map(|&sample| (f32::from(sample) / f32::from(u16::MAX)).to_le_bytes());
    let locations = map.raster.cells().iter().flat_map(|location| location.0.to_le_bytes());
    let borders = cell_borders(map)
      .into_iter()
      .flat_map(|borders| [borders.east.to_texel(), borders.south.to_texel()]);

    Some(Self {
      heights: texture(map.heights.width(), map.heights.height(), Format::RF, heights)?,
      locations: texture(map.raster.width(), map.raster.height(), Format::RG8, locations)?,
      borders: texture(map.raster.width(), map.raster.height(), Format::RG8, borders)?,
    })
  }
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

fn texture(width: u32, height: u32, format: Format, bytes: impl Iterator<Item = u8>) -> Option<Gd<ImageTexture>> {
  let data: PackedByteArray = bytes.collect();
  let image = Image::create_from_data(width as i32, height as i32, false, format, &data)?;

  ImageTexture::create_from_image(&image)
}

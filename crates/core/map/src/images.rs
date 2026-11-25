//! Reading the map's source images.

use png::{BitDepth, ColorType, Decoder, Transformations};

/// An image's pixels as one 16-bit sample per channel.
pub struct Pixels {
  pub width: u32,
  pub height: u32,
  pub channels: usize,
  pub samples: Vec<u16>,
}

impl Pixels {
  /// The first three channels of each pixel, at 8 bits each.
  pub fn to_rgb8(&self) -> Vec<[u8; 3]> {
    self
      .samples
      .chunks_exact(self.channels)
      .map(|pixel| {
        // Grey images repeat their one channel.
        let channel = |index: usize| (pixel[index.min(self.color_channels() - 1)] >> 8) as u8;

        [channel(0), channel(1), channel(2)]
      })
      .collect()
  }

  /// The first channel of each pixel.
  pub fn to_grey16(&self) -> Vec<u16> {
    self.samples.chunks_exact(self.channels).map(|pixel| pixel[0]).collect()
  }

  fn color_channels(&self) -> usize {
    if self.channels >= 3 { 3 } else { 1 }
  }
}

/// Decodes a PNG file, whatever its colour type and depth.
pub fn decode_png(bytes: &[u8]) -> Option<Pixels> {
  let mut decoder = Decoder::new(bytes);
  decoder.set_transformations(Transformations::EXPAND);

  let mut reader = decoder.read_info().ok()?;
  let mut buffer = vec![0; reader.output_buffer_size()];
  let frame = reader.next_frame(&mut buffer).ok()?;
  let bytes = &buffer[..frame.buffer_size()];

  let channels = match frame.color_type {
    ColorType::Grayscale => 1,
    ColorType::GrayscaleAlpha => 2,
    ColorType::Rgb => 3,
    ColorType::Rgba => 4,
    ColorType::Indexed => return None,
  };

  let samples = match frame.bit_depth {
    BitDepth::Eight => bytes.iter().map(|&byte| u16::from(byte) * 257).collect(),
    BitDepth::Sixteen => bytes
      .chunks_exact(2)
      .map(|pair| u16::from_be_bytes([pair[0], pair[1]]))
      .collect(),
    _ => return None,
  };

  Some(Pixels {
    width: frame.width,
    height: frame.height,
    channels,
    samples,
  })
}

#[cfg(test)]
mod tests {
  use png::{BitDepth, ColorType, Encoder};

  use super::*;

  fn encode(width: u32, height: u32, color: ColorType, depth: BitDepth, data: &[u8]) -> Vec<u8> {
    let mut bytes = Vec::new();
    let mut encoder = Encoder::new(&mut bytes, width, height);
    encoder.set_color(color);
    encoder.set_depth(depth);
    encoder.write_header().unwrap().write_image_data(data).unwrap();
    bytes
  }

  #[test]
  fn a_colour_image_keeps_its_colours() {
    let png = encode(2, 1, ColorType::Rgb, BitDepth::Eight, &[10, 20, 30, 40, 50, 60]);

    let pixels = decode_png(&png).unwrap();

    assert_eq!((pixels.width, pixels.height), (2, 1));
    assert_eq!(pixels.to_rgb8(), [[10, 20, 30], [40, 50, 60]]);
  }

  #[test]
  fn a_sixteen_bit_grey_image_keeps_its_full_depth() {
    let png = encode(2, 1, ColorType::Grayscale, BitDepth::Sixteen, &[0x12, 0x34, 0xff, 0xff]);

    let pixels = decode_png(&png).unwrap();

    assert_eq!(pixels.to_grey16(), [0x1234, 0xffff]);
  }

  #[test]
  fn an_eight_bit_grey_image_spans_the_full_range() {
    let png = encode(2, 1, ColorType::Grayscale, BitDepth::Eight, &[0, 255]);

    let pixels = decode_png(&png).unwrap();

    assert_eq!(pixels.to_grey16(), [0, u16::MAX]);
  }

  #[test]
  fn bytes_that_are_not_an_image_do_not_decode() {
    assert!(decode_png(b"not a png").is_none());
  }
}

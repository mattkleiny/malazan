//! The heightfield: how high the ground stands across the map.

/// A grid of ground heights stretched over the whole map, wrapping east to
/// west. It need not share the location raster's resolution.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Heightfield {
  width: u32,
  height: u32,
  samples: Vec<u16>,
}

impl Heightfield {
  /// Builds a heightfield from rows of samples, north-west corner first.
  pub fn from_samples(width: u32, height: u32, samples: Vec<u16>) -> Option<Self> {
    if width == 0 || height == 0 || samples.len() != width as usize * height as usize {
      return None;
    }

    Some(Self { width, height, samples })
  }

  pub fn width(&self) -> u32 {
    self.width
  }

  pub fn height(&self) -> u32 {
    self.height
  }

  pub fn samples(&self) -> &[u16] {
    &self.samples
  }

  /// The height (0..1) at a point given as a share of the map's width and
  /// height, blended between the nearest samples.
  pub fn height_at(&self, u: f64, v: f64) -> f64 {
    let x = u * f64::from(self.width) - 0.5;
    let y = (v * f64::from(self.height) - 0.5).clamp(0.0, f64::from(self.height - 1));

    let (column, row) = (x.floor(), y.floor());
    let (blend_x, blend_y) = (x - column, y - row);
    let (column, row) = (column as i64, row as i64);

    let north = lerp(self.sample(column, row), self.sample(column + 1, row), blend_x);
    let south = lerp(self.sample(column, row + 1), self.sample(column + 1, row + 1), blend_x);

    lerp(north, south, blend_y)
  }

  fn sample(&self, column: i64, row: i64) -> f64 {
    let column = column.rem_euclid(i64::from(self.width)) as usize;
    let row = row.clamp(0, i64::from(self.height) - 1) as usize;

    f64::from(self.samples[row * self.width as usize + column]) / f64::from(u16::MAX)
  }
}

fn lerp(from: f64, to: f64, amount: f64) -> f64 {
  from + (to - from) * amount
}

#[cfg(test)]
mod tests {
  use super::*;

  /// Two samples wide and one tall: lowest in the west, highest in the east.
  fn ramp() -> Heightfield {
    Heightfield::from_samples(2, 1, vec![0, u16::MAX]).unwrap()
  }

  #[test]
  fn a_point_at_a_sample_has_that_sample_s_height() {
    let heights = ramp();

    assert_eq!(heights.height_at(0.25, 0.5), 0.0);
    assert_eq!(heights.height_at(0.75, 0.5), 1.0);
  }

  #[test]
  fn a_point_between_samples_blends_them() {
    let heights = ramp();

    assert!((heights.height_at(0.5, 0.5) - 0.5).abs() < 1e-9);
  }

  #[test]
  fn heights_blend_across_the_seam() {
    let heights = ramp();

    assert!((heights.height_at(0.0, 0.5) - 0.5).abs() < 1e-9);
    assert!((heights.height_at(1.0, 0.5) - 0.5).abs() < 1e-9);
  }

  #[test]
  fn samples_that_do_not_fill_the_grid_are_refused() {
    assert!(Heightfield::from_samples(2, 2, vec![0; 3]).is_none());
  }
}

//! Map editing: the changes a designer makes to a map, and writing it back out.

pub mod brushes;
pub mod definitions;
pub mod files;
pub mod patches;

/// A block of cells on a grid (the location raster or the heightfield),
/// north-west corner first.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct CellRect {
  pub column: u32,
  pub row: u32,
  pub width: u32,
  pub height: u32,
}

impl CellRect {
  /// The cells a circle reaches on a grid, cut to the grid; none when the
  /// circle misses it. The centre and radius are in cells.
  pub fn around(centre_x: f64, centre_y: f64, radius: f64, grid_width: u32, grid_height: u32) -> Option<CellRect> {
    let west = (centre_x - radius).floor().max(0.0);
    let north = (centre_y - radius).floor().max(0.0);
    let east = (centre_x + radius).ceil().min(f64::from(grid_width));
    let south = (centre_y + radius).ceil().min(f64::from(grid_height));

    if !(west < east && north < south) {
      return None;
    }

    Some(CellRect {
      column: west as u32,
      row: north as u32,
      width: (east - west) as u32,
      height: (south - north) as u32,
    })
  }

  /// The smallest block holding both.
  pub fn union(self, other: CellRect) -> CellRect {
    let column = self.column.min(other.column);
    let row = self.row.min(other.row);
    let east = (self.column + self.width).max(other.column + other.width);
    let south = (self.row + self.height).max(other.row + other.height);

    CellRect {
      column,
      row,
      width: east - column,
      height: south - row,
    }
  }

  /// The block with `by` more cells to its west and north, cut to the grid.
  pub fn grown_north_west(self, by: u32) -> CellRect {
    let column = self.column.saturating_sub(by);
    let row = self.row.saturating_sub(by);

    CellRect {
      column,
      row,
      width: self.width + (self.column - column),
      height: self.height + (self.row - row),
    }
  }

  /// Every cell of the block, row by row.
  pub fn cells(self) -> impl Iterator<Item = (u32, u32)> {
    (self.row..self.row + self.height)
      .flat_map(move |row| (self.column..self.column + self.width).map(move |column| (column, row)))
  }

  pub fn area(self) -> usize {
    self.width as usize * self.height as usize
  }
}

/// Grows an optional block to take in another.
pub fn widen(rect: Option<CellRect>, by: Option<CellRect>) -> Option<CellRect> {
  match (rect, by) {
    (Some(rect), Some(by)) => Some(rect.union(by)),
    (rect, by) => rect.or(by),
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn a_circle_reaches_the_cells_it_overlaps() {
    let rect = CellRect::around(5.5, 5.5, 1.0, 10, 10).unwrap();

    assert_eq!(rect, CellRect {
      column: 4,
      row: 4,
      width: 3,
      height: 3
    });
  }

  #[test]
  fn a_circle_over_the_edge_is_cut_to_the_grid() {
    let rect = CellRect::around(0.5, 9.5, 2.0, 10, 10).unwrap();

    assert_eq!(rect, CellRect {
      column: 0,
      row: 7,
      width: 3,
      height: 3
    });
  }

  #[test]
  fn a_circle_off_the_grid_reaches_nothing() {
    assert_eq!(CellRect::around(-5.0, 5.0, 1.0, 10, 10), None);
  }

  #[test]
  fn two_blocks_join_into_one_holding_both() {
    let a = CellRect {
      column: 1,
      row: 1,
      width: 2,
      height: 2,
    };
    let b = CellRect {
      column: 4,
      row: 0,
      width: 1,
      height: 1,
    };

    assert_eq!(a.union(b), CellRect {
      column: 1,
      row: 0,
      width: 4,
      height: 3
    });
  }

  #[test]
  fn growing_stops_at_the_grid_s_corner() {
    let rect = CellRect {
      column: 0,
      row: 3,
      width: 2,
      height: 2,
    };

    assert_eq!(rect.grown_north_west(1), CellRect {
      column: 0,
      row: 2,
      width: 2,
      height: 3
    });
  }
}

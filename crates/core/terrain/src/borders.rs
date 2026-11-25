//! Borders: the lines drawn where locations and the tiers above them meet.

use game_map::{LocationId, Map};

/// How much a border divides, from none between cells of one location to the
/// most between continents or between land and water.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum BorderRank {
  None,
  Location,
  Province,
  Area,
  Region,
  Continent,
}

impl BorderRank {
  pub const HIGHEST: BorderRank = BorderRank::Continent;

  /// The rank as a texture stores it: evenly spaced up to the highest at 255.
  pub fn to_texel(self) -> u8 {
    self as u8 * (u8::MAX / Self::HIGHEST as u8)
  }
}

/// The rank of the border between two locations: the highest tier they do not
/// share. A shore ranks highest.
fn rank_between(map: &Map, a: LocationId, b: LocationId) -> BorderRank {
  if a == b {
    return BorderRank::None;
  }

  let (Some(from), Some(to)) = (map.location(a), map.location(b)) else {
    return BorderRank::None;
  };

  if from.terrain.is_water() != to.terrain.is_water() {
    return BorderRank::HIGHEST;
  }

  if from.province == to.province {
    return BorderRank::Location;
  }

  let (Some(from), Some(to)) = (map.province_of(a), map.province_of(b)) else {
    return BorderRank::Province;
  };

  if from.area == to.area {
    return BorderRank::Province;
  }

  let (Some(from), Some(to)) = (map.area_of(a), map.area_of(b)) else {
    return BorderRank::Area;
  };

  if from.region == to.region {
    return BorderRank::Area;
  }

  let (Some(from), Some(to)) = (map.region_of(a), map.region_of(b)) else {
    return BorderRank::Region;
  };

  if from.continent == to.continent {
    BorderRank::Region
  } else {
    BorderRank::Continent
  }
}

/// The borders on a cell's eastern and southern sides.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct CellBorders {
  pub east: BorderRank,
  pub south: BorderRank,
}

/// The borders of each cell of the location raster, row by row. The map's
/// own edges are no border.
pub fn cell_borders(map: &Map) -> Vec<CellBorders> {
  let raster = &map.raster;
  let mut borders = Vec::with_capacity(raster.cells().len());

  for row in 0..i64::from(raster.height()) {
    for column in 0..i64::from(raster.width()) {
      borders.push(borders_of(map, column, row));
    }
  }

  borders
}

/// The borders on one cell's eastern and southern sides.
pub fn borders_of(map: &Map, column: i64, row: i64) -> CellBorders {
  let raster = &map.raster;
  let rank_towards = |neighbour: Option<LocationId>| match (raster.cell(column, row), neighbour) {
    (Some(here), Some(there)) => rank_between(map, here, there),
    _ => BorderRank::None,
  };

  CellBorders {
    east: rank_towards(raster.cell(column + 1, row)),
    south: rank_towards(raster.cell(column, row + 1)),
  }
}

#[cfg(test)]
mod tests {
  use game_map::{Heightfield, MapParts};

  use super::*;

  /// A map one row tall with one location per cell, west to east, each given
  /// as its province, area, region, continent and terrain.
  fn strip(cells: &[[&str; 5]]) -> Map {
    let definitions: String = cells
      .iter()
      .enumerate()
      .map(|(index, [province, area, region, continent, terrain])| {
        format!("{index},0,0,L{index},{province},{area},{region},{continent},{terrain}\n")
      })
      .collect();
    let key_colors: Vec<[u8; 3]> = (0..cells.len()).map(|index| [index as u8, 0, 0]).collect();

    Map::from_parts(MapParts {
      definitions: &definitions,
      width: cells.len() as u32,
      height: 1,
      key_colors: &key_colors,
      heights: Heightfield::from_samples(1, 1, vec![0]).expect("one sample fills a one by one grid"),
      sea_level: 0.5,
    })
    .unwrap()
  }

  #[test]
  fn a_border_ranks_by_the_highest_tier_it_divides() {
    let map = strip(&[
      ["P1", "A1", "R1", "C1", "plains"],
      ["P1", "A1", "R1", "C1", "plains"],
      ["P2", "A1", "R1", "C1", "plains"],
      ["P3", "A2", "R1", "C1", "plains"],
      ["P4", "A3", "R2", "C1", "plains"],
      ["P5", "A4", "R3", "C2", "plains"],
    ]);

    let east: Vec<BorderRank> = cell_borders(&map).iter().map(|borders| borders.east).collect();

    assert_eq!(east, [
      BorderRank::Location,
      BorderRank::Province,
      BorderRank::Area,
      BorderRank::Region,
      BorderRank::Continent,
      BorderRank::None,
    ]);
  }

  #[test]
  fn a_shore_ranks_highest() {
    let map = strip(&[["P1", "A1", "R1", "C1", "plains"], ["P1", "A1", "R1", "C1", "sea"], [
      "P1", "A1", "R1", "C1", "sea",
    ]]);

    let east: Vec<BorderRank> = cell_borders(&map).iter().map(|borders| borders.east).collect();

    assert_eq!(east, [BorderRank::Continent, BorderRank::Location, BorderRank::None]);
  }

  #[test]
  fn cells_inside_one_location_lie_on_no_border() {
    let definitions = "1,0,0,Only,P,A,R,C,plains";
    let map = Map::from_parts(MapParts {
      definitions,
      width: 2,
      height: 2,
      key_colors: &[[1, 0, 0]; 4],
      heights: Heightfield::from_samples(1, 1, vec![0]).expect("one sample fills a one by one grid"),
      sea_level: 0.5,
    })
    .unwrap();

    let none = CellBorders {
      east: BorderRank::None,
      south: BorderRank::None,
    };

    assert_eq!(cell_borders(&map), [none; 4]);
  }

  #[test]
  fn the_southern_edge_of_the_map_is_no_border() {
    let map = strip(&[["P1", "A1", "R1", "C1", "plains"], ["P2", "A1", "R1", "C1", "plains"]]);

    assert!(
      cell_borders(&map)
        .iter()
        .all(|borders| borders.south == BorderRank::None)
    );
  }

  #[test]
  fn ranks_spread_evenly_over_a_texel() {
    assert_eq!(BorderRank::None.to_texel(), 0);
    assert_eq!(BorderRank::Continent.to_texel(), 255);
  }
}

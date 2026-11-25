//! The map modes that show the map itself: its terrain and its divisions.

use crate::{
  locations::{Location, LocationId, Terrain},
  map::Map,
  modes::{Fill, MapMode},
  selections::{Selection, Tier, division_selection},
};

/// Colours each location by the kind of ground it is.
pub struct TerrainMode;

impl MapMode for TerrainMode {
  fn key(&self) -> &'static str {
    "terrain"
  }

  fn label(&self) -> &'static str {
    "Terrain"
  }

  fn fills(&self, map: &Map) -> Vec<Fill> {
    land_fills(map, |location| Some(terrain_color(location.terrain)))
  }
}

/// Colours each location, sea included, in its own key colour.
pub struct LocationMode;

impl MapMode for LocationMode {
  fn key(&self) -> &'static str {
    "locations"
  }

  fn label(&self) -> &'static str {
    "Locations"
  }

  fn fills(&self, map: &Map) -> Vec<Fill> {
    map
      .locations
      .iter()
      .map(|location| Fill::solid(location.key_color))
      .collect()
  }
}

/// Colours each location by its province.
pub struct ProvinceMode;

impl MapMode for ProvinceMode {
  fn key(&self) -> &'static str {
    "provinces"
  }

  fn label(&self) -> &'static str {
    "Provinces"
  }

  fn fills(&self, map: &Map) -> Vec<Fill> {
    land_fills(map, |location| Some(division_color(1, location.province.0)))
  }

  fn select(&self, map: &Map, location: LocationId) -> Option<Selection> {
    division_selection(map, location, Tier::Province)
  }
}

/// Colours each location by its area.
pub struct AreaMode;

impl MapMode for AreaMode {
  fn key(&self) -> &'static str {
    "areas"
  }

  fn label(&self) -> &'static str {
    "Areas"
  }

  fn fills(&self, map: &Map) -> Vec<Fill> {
    land_fills(map, |location| {
      let province = map.provinces.get(usize::from(location.province.0))?;

      Some(division_color(2, province.area.0))
    })
  }

  fn select(&self, map: &Map, location: LocationId) -> Option<Selection> {
    division_selection(map, location, Tier::Area)
  }
}

/// Colours each location by its region.
pub struct RegionMode;

impl MapMode for RegionMode {
  fn key(&self) -> &'static str {
    "regions"
  }

  fn label(&self) -> &'static str {
    "Regions"
  }

  fn fills(&self, map: &Map) -> Vec<Fill> {
    land_fills(map, |location| {
      let province = map.provinces.get(usize::from(location.province.0))?;
      let area = map.areas.get(usize::from(province.area.0))?;

      Some(division_color(3, area.region.0))
    })
  }

  fn select(&self, map: &Map, location: LocationId) -> Option<Selection> {
    division_selection(map, location, Tier::Region)
  }
}

/// Colours each location by its continent.
pub struct ContinentMode;

impl MapMode for ContinentMode {
  fn key(&self) -> &'static str {
    "continents"
  }

  fn label(&self) -> &'static str {
    "Continents"
  }

  fn fills(&self, map: &Map) -> Vec<Fill> {
    land_fills(map, |location| {
      let province = map.provinces.get(usize::from(location.province.0))?;
      let area = map.areas.get(usize::from(province.area.0))?;
      let region = map.regions.get(usize::from(area.region.0))?;

      Some(division_color(4, region.continent.0))
    })
  }

  fn select(&self, map: &Map, location: LocationId) -> Option<Selection> {
    division_selection(map, location, Tier::Continent)
  }
}

crate::map_mode!(order: 10, TerrainMode);
crate::map_mode!(order: 20, LocationMode);
crate::map_mode!(order: 30, ProvinceMode);
crate::map_mode!(order: 40, AreaMode);
crate::map_mode!(order: 50, RegionMode);
crate::map_mode!(order: 60, ContinentMode);

/// Fills every land location with the colour `color_of` gives it, leaving
/// water clear so the sea shows through.
pub fn land_fills(map: &Map, color_of: impl Fn(&Location) -> Option<[u8; 3]>) -> Vec<Fill> {
  map
    .locations
    .iter()
    .map(|location| {
      if location.terrain.is_water() {
        return Fill::CLEAR;
      }

      color_of(location).map_or(Fill::CLEAR, Fill::solid)
    })
    .collect()
}

/// A colour for a division that stays the same from run to run and differs
/// from its neighbours in the table. `tier` sets divisions of different
/// tiers apart.
pub fn division_color(tier: u32, id: u16) -> [u8; 3] {
  // Stepping the hue by the golden ratio keeps consecutive ids far apart.
  let hue = (f64::from(id) * 0.618_033_988_75 + f64::from(tier) * 0.17).fract();
  let saturation = 0.45 + 0.2 * f64::from(id % 3) / 2.0;
  let value = 0.7 + 0.2 * f64::from(id % 4) / 3.0;

  hsv_to_rgb(hue, saturation, value)
}

fn terrain_color(terrain: Terrain) -> [u8; 3] {
  match terrain {
    Terrain::Plains => [134, 160, 84],
    Terrain::Forest => [62, 110, 62],
    Terrain::Hills => [150, 142, 96],
    Terrain::Mountains => [128, 120, 114],
    Terrain::Desert => [214, 190, 134],
    Terrain::Marsh => [88, 116, 100],
    Terrain::Tundra => [196, 204, 204],
    Terrain::Sea => [44, 84, 128],
    Terrain::Lake => [70, 122, 160],
  }
}

/// Converts a colour from hue, saturation and value (each 0..1).
fn hsv_to_rgb(hue: f64, saturation: f64, value: f64) -> [u8; 3] {
  let sector = (hue * 6.0).rem_euclid(6.0);
  let chroma = value * saturation;
  let second = chroma * (1.0 - (sector % 2.0 - 1.0).abs());

  let [red, green, blue] = match sector as u32 {
    0 => [chroma, second, 0.0],
    1 => [second, chroma, 0.0],
    2 => [0.0, chroma, second],
    3 => [0.0, second, chroma],
    4 => [second, 0.0, chroma],
    _ => [chroma, 0.0, second],
  };

  [red, green, blue].map(|channel| ((channel + value - chroma) * 255.0).round() as u8)
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::{heights::Heightfield, locations::LocationId, map::MapParts};

  /// Two land locations sharing a province, a third in another, and a sea.
  fn map() -> Map {
    Map::from_parts(MapParts {
      definitions: "\
1,0,0,A,P1,A1,R1,C1,plains
2,0,0,B,P1,A1,R1,C1,forest
3,0,0,C,P2,A1,R1,C1,plains
4,0,0,D,P3,A1,R1,C1,sea",
      width: 4,
      height: 1,
      key_colors: &[[1, 0, 0], [2, 0, 0], [3, 0, 0], [4, 0, 0]],
      heights: Heightfield::from_samples(1, 1, vec![0]).expect("one sample fills a one by one grid"),
      sea_level: 0.5,
    })
    .unwrap()
  }

  const SEA: LocationId = LocationId(3);

  #[test]
  fn locations_of_one_province_share_its_colour() {
    let fills = ProvinceMode.fills(&map());

    assert_eq!(fills[0], fills[1]);
    assert_ne!(fills[0], fills[2]);
  }

  #[test]
  fn terrain_mode_colours_by_the_kind_of_ground() {
    let fills = TerrainMode.fills(&map());

    assert_eq!(fills[0], fills[2]);
    assert_ne!(fills[0], fills[1]);
  }

  #[test]
  fn the_sea_is_left_clear_by_modes_that_colour_land() {
    let map = map();

    assert_eq!(ProvinceMode.fills(&map)[SEA.index()], Fill::CLEAR);
    assert_eq!(ContinentMode.fills(&map)[SEA.index()], Fill::CLEAR);
  }

  #[test]
  fn location_mode_colours_the_sea_too() {
    assert_eq!(LocationMode.fills(&map())[SEA.index()], Fill::solid([4, 0, 0]));
  }

  #[test]
  fn each_division_mode_selects_its_own_tier() {
    let map = map();

    assert_eq!(TerrainMode.select(&map, LocationId(0)).unwrap().kind, "location");
    assert_eq!(ProvinceMode.select(&map, LocationId(0)).unwrap().kind, "province");
    assert_eq!(AreaMode.select(&map, LocationId(0)).unwrap().kind, "area");
    assert_eq!(RegionMode.select(&map, LocationId(0)).unwrap().kind, "region");
    assert_eq!(ContinentMode.select(&map, LocationId(0)).unwrap().kind, "continent");
  }

  #[test]
  fn pure_hues_convert_to_their_primaries() {
    assert_eq!(hsv_to_rgb(0.0, 1.0, 1.0), [255, 0, 0]);
    assert_eq!(hsv_to_rgb(1.0 / 3.0, 1.0, 1.0), [0, 255, 0]);
    assert_eq!(hsv_to_rgb(2.0 / 3.0, 1.0, 1.0), [0, 0, 255]);
    assert_eq!(hsv_to_rgb(0.5, 0.0, 0.5), [128, 128, 128]);
  }
}

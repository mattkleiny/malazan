//! Selections: what picking the map chooses, which the map mode decides.

use crate::{
  divisions::{AreaId, ContinentId, ProvinceId, RegionId},
  locations::LocationId,
  map::Map,
};

/// A thing chosen on the map: a location, a division, or anything else a map
/// mode offers, such as the wealth of an area.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Selection {
  pub kind: &'static str, // What sort of thing it is, such as "region".
  pub id: u64,            // Which one it is among its kind.
  pub title: String,
  pub details: Vec<String>,       // Lines describing it to the player.
  pub locations: Vec<LocationId>, // Where it lies on the map, lit while hovered or selected.
}

impl Selection {
  /// Whether two selections choose the same thing, whatever they say about it.
  pub fn is_same(&self, other: &Selection) -> bool {
    self.kind == other.kind && self.id == other.id
  }
}

/// A tier of the map's divisions.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Tier {
  Province,
  Area,
  Region,
  Continent,
}

impl Tier {
  fn key(self) -> &'static str {
    match self {
      Tier::Province => "province",
      Tier::Area => "area",
      Tier::Region => "region",
      Tier::Continent => "continent",
    }
  }
}

/// The divisions a location sits in, one per tier.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
struct Divisions {
  province: ProvinceId,
  area: AreaId,
  region: RegionId,
  continent: ContinentId,
}

impl Divisions {
  fn of(map: &Map, location: LocationId) -> Option<Divisions> {
    let province = map.location(location)?.province;
    let area = map.provinces.get(usize::from(province.0))?.area;
    let region = map.areas.get(usize::from(area.0))?.region;
    let continent = map.regions.get(usize::from(region.0))?.continent;

    Some(Divisions {
      province,
      area,
      region,
      continent,
    })
  }

  fn id(self, tier: Tier) -> u16 {
    match tier {
      Tier::Province => self.province.0,
      Tier::Area => self.area.0,
      Tier::Region => self.region.0,
      Tier::Continent => self.continent.0,
    }
  }

  /// The names of this tier's division and those above it, smallest first.
  fn names_from(self, map: &Map, tier: Tier) -> Vec<&str> {
    let names = [
      map
        .provinces
        .get(usize::from(self.province.0))
        .map(|it| it.name.as_str()),
      map.areas.get(usize::from(self.area.0)).map(|it| it.name.as_str()),
      map.regions.get(usize::from(self.region.0)).map(|it| it.name.as_str()),
      map
        .continents
        .get(usize::from(self.continent.0))
        .map(|it| it.name.as_str()),
    ];

    names[tier as usize..].iter().flatten().copied().collect()
  }
}

/// Selects one location.
pub fn location_selection(map: &Map, location: LocationId) -> Option<Selection> {
  let found = map.location(location)?;
  let divisions = Divisions::of(map, location)?;

  Some(Selection {
    kind: "location",
    id: u64::from(location.0),
    title: found.name.clone(),
    details: vec![
      found.terrain.label().to_string(),
      divisions.names_from(map, Tier::Province).join(" · "),
    ],
    locations: vec![location],
  })
}

/// Selects the whole division of a tier that a location sits in.
pub fn division_selection(map: &Map, location: LocationId, tier: Tier) -> Option<Selection> {
  let divisions = Divisions::of(map, location)?;
  let id = divisions.id(tier);

  let locations: Vec<LocationId> = (0..map.locations.len())
    .map(|index| LocationId(index as u16))
    .filter(|&other| Divisions::of(map, other).is_some_and(|it| it.id(tier) == id))
    .collect();

  let mut names = divisions.names_from(map, tier).into_iter();
  let title = names.next()?.to_string();
  let above: Vec<&str> = names.collect();

  let mut details = vec![format!("{} {}", locations.len(), plural(locations.len(), "location"))];
  if !above.is_empty() {
    details.push(above.join(" · "));
  }

  Some(Selection {
    kind: tier.key(),
    id: u64::from(id),
    title,
    details,
    locations,
  })
}

fn plural(count: usize, noun: &str) -> String {
  if count == 1 {
    noun.to_string()
  } else {
    format!("{noun}s")
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::{heights::Heightfield, map::MapParts};

  /// Four locations: two in one province, one in another of the same area,
  /// and one in another region.
  fn map() -> Map {
    Map::from_parts(MapParts {
      definitions: "\
1,0,0,Aren,Coast,Raraku,Seven Cities,Quon Tali,plains
2,0,0,Ehrlitan,Coast,Raraku,Seven Cities,Quon Tali,desert
3,0,0,Pan'potsun,Hills,Raraku,Seven Cities,Quon Tali,hills
4,0,0,Unta,Heartland,Kartool,Quon,Quon Tali,plains",
      width: 4,
      height: 1,
      key_colors: &[[1, 0, 0], [2, 0, 0], [3, 0, 0], [4, 0, 0]],
      heights: Heightfield::from_samples(1, 1, vec![0]).expect("one sample fills a one by one grid"),
      sea_level: 0.5,
    })
    .unwrap()
  }

  #[test]
  fn a_location_selection_is_just_that_location() {
    let selection = location_selection(&map(), LocationId(1)).unwrap();

    assert_eq!(selection.title, "Ehrlitan");
    assert_eq!(selection.locations, [LocationId(1)]);
    assert_eq!(selection.details, [
      "Desert",
      "Coast · Raraku · Seven Cities · Quon Tali"
    ]);
  }

  #[test]
  fn a_province_selection_takes_in_every_location_of_the_province() {
    let selection = division_selection(&map(), LocationId(0), Tier::Province).unwrap();

    assert_eq!(selection.title, "Coast");
    assert_eq!(selection.locations, [LocationId(0), LocationId(1)]);
    assert_eq!(selection.details, ["2 locations", "Raraku · Seven Cities · Quon Tali"]);
  }

  #[test]
  fn a_region_selection_takes_in_every_location_of_the_region() {
    let selection = division_selection(&map(), LocationId(2), Tier::Region).unwrap();

    assert_eq!(selection.title, "Seven Cities");
    assert_eq!(selection.locations, [LocationId(0), LocationId(1), LocationId(2)]);
  }

  #[test]
  fn picking_anywhere_in_a_division_selects_the_same_thing() {
    let map = map();

    let from_aren = division_selection(&map, LocationId(0), Tier::Area).unwrap();
    let from_hills = division_selection(&map, LocationId(2), Tier::Area).unwrap();
    let from_unta = division_selection(&map, LocationId(3), Tier::Area).unwrap();

    assert!(from_aren.is_same(&from_hills));
    assert!(!from_aren.is_same(&from_unta));
  }

  #[test]
  fn a_continent_names_nothing_above_it() {
    let selection = division_selection(&map(), LocationId(3), Tier::Continent).unwrap();

    assert_eq!(selection.details, ["4 locations"]);
  }

  #[test]
  fn nothing_is_selected_for_a_location_the_map_does_not_have() {
    assert!(location_selection(&map(), LocationId(9)).is_none());
    assert!(division_selection(&map(), LocationId(9), Tier::Region).is_none());
  }
}

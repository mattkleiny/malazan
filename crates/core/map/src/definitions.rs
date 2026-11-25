//! Map definitions: the table naming every location and the tiers it sits in.

use std::collections::HashMap;

use crate::{
  divisions::{Area, AreaId, Continent, ContinentId, Province, ProvinceId, Region, RegionId},
  locations::{Location, Terrain},
  map::MapError,
};

/// The columns of a definitions row, in order.
const COLUMNS: usize = 9;

/// Everything the definitions table names.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct Definitions {
  pub locations: Vec<Location>,
  pub provinces: Vec<Province>,
  pub areas: Vec<Area>,
  pub regions: Vec<Region>,
  pub continents: Vec<Continent>,
}

/// One tier's divisions as they are read, each with the division above it.
#[derive(Default)]
struct Tier {
  rows: Vec<(String, u16)>,
  by_name: HashMap<String, u16>,
}

impl Tier {
  /// The id of the division called `name`, added if it is new. A division
  /// already placed under a different parent is an error.
  fn place(&mut self, name: &str, parent: u16, line: usize) -> Result<u16, MapError> {
    if let Some(&id) = self.by_name.get(name) {
      if self.rows[usize::from(id)].1 != parent {
        return Err(MapError::SplitDivision {
          line,
          name: name.to_string(),
        });
      }

      return Ok(id);
    }

    let id = u16::try_from(self.rows.len()).map_err(|_| MapError::TooManyLocations)?;

    self.rows.push((name.to_string(), parent));
    self.by_name.insert(name.to_string(), id);

    Ok(id)
  }
}

/// Reads a definitions table: one comma-separated row per location, giving
/// `red,green,blue,location,province,area,region,continent,terrain`. Blank
/// lines, `#` comments and a header row are skipped.
pub fn parse_definitions(text: &str) -> Result<Definitions, MapError> {
  let mut locations = Vec::new();
  let mut colors = HashMap::new();
  let mut names = HashMap::new();
  let (mut provinces, mut areas, mut regions, mut continents) =
    (Tier::default(), Tier::default(), Tier::default(), Tier::default());

  for (index, row) in text.lines().enumerate() {
    let line = index + 1;
    let row = row.trim();

    if row.is_empty() || row.starts_with('#') || row.starts_with("red,") {
      continue;
    }

    let fields: Vec<&str> = row.split(',').map(str::trim).collect();

    if fields.len() != COLUMNS || fields.iter().any(|field| field.is_empty()) {
      return Err(MapError::MalformedRow { line });
    }

    let channel = |field: &str| field.parse::<u8>().map_err(|_| MapError::MalformedRow { line });
    let key_color = [channel(fields[0])?, channel(fields[1])?, channel(fields[2])?];
    let name = fields[3];

    let terrain = Terrain::from_key(fields[8]).ok_or_else(|| MapError::UnknownTerrain {
      line,
      terrain: fields[8].to_string(),
    })?;

    if colors.insert(key_color, line).is_some() {
      return Err(MapError::DuplicateColor { line, color: key_color });
    }

    if names.insert(name.to_string(), line).is_some() {
      return Err(MapError::DuplicateLocation {
        line,
        name: name.to_string(),
      });
    }

    // The id one past the last is kept free to mean no location.
    if locations.len() >= usize::from(u16::MAX) {
      return Err(MapError::TooManyLocations);
    }

    let continent = continents.place(fields[7], 0, line)?;
    let region = regions.place(fields[6], continent, line)?;
    let area = areas.place(fields[5], region, line)?;
    let province = provinces.place(fields[4], area, line)?;

    locations.push(Location {
      name: name.to_string(),
      key_color,
      province: ProvinceId(province),
      terrain,
    });
  }

  if locations.is_empty() {
    return Err(MapError::NoLocations);
  }

  Ok(Definitions {
    locations,
    provinces: provinces
      .rows
      .into_iter()
      .map(|(name, area)| Province {
        name,
        area: AreaId(area),
      })
      .collect(),
    areas: areas
      .rows
      .into_iter()
      .map(|(name, region)| Area {
        name,
        region: RegionId(region),
      })
      .collect(),
    regions: regions
      .rows
      .into_iter()
      .map(|(name, continent)| Region {
        name,
        continent: ContinentId(continent),
      })
      .collect(),
    continents: continents
      .rows
      .into_iter()
      .map(|(name, _)| Continent { name })
      .collect(),
  })
}

#[cfg(test)]
mod tests {
  use super::*;

  const TABLE: &str = "\
red,green,blue,location,province,area,region,continent,terrain
# The north.
10,0,0,Aren,Seven Cities Coast,Raraku,Seven Cities,Quon Tali,desert
20,0,0,Ehrlitan,Seven Cities Coast,Raraku,Seven Cities,Quon Tali,plains

30,0,0,Otataral Sea,Otataral Waters,Raraku,Seven Cities,Quon Tali,sea
";

  #[test]
  fn each_row_names_a_location() {
    let definitions = parse_definitions(TABLE).unwrap();

    assert_eq!(definitions.locations.len(), 3);
    assert_eq!(definitions.locations[1].name, "Ehrlitan");
    assert_eq!(definitions.locations[1].key_color, [20, 0, 0]);
    assert_eq!(definitions.locations[2].terrain, Terrain::Sea);
  }

  #[test]
  fn locations_naming_the_same_province_share_it() {
    let definitions = parse_definitions(TABLE).unwrap();

    assert_eq!(definitions.provinces.len(), 2);
    assert_eq!(definitions.locations[0].province, definitions.locations[1].province);
    assert_eq!(definitions.areas.len(), 1);
    assert_eq!(definitions.continents[0].name, "Quon Tali");
  }

  #[test]
  fn a_province_placed_in_two_areas_is_refused() {
    let table = "1,0,0,A,Coast,North,Realm,World,plains\n2,0,0,B,Coast,South,Realm,World,plains";

    assert_eq!(
      parse_definitions(table),
      Err(MapError::SplitDivision {
        line: 2,
        name: "Coast".to_string()
      })
    );
  }

  #[test]
  fn two_locations_never_share_a_key_colour() {
    let table = "1,0,0,A,P,A,R,C,plains\n1,0,0,B,P,A,R,C,plains";

    assert_eq!(
      parse_definitions(table),
      Err(MapError::DuplicateColor {
        line: 2,
        color: [1, 0, 0]
      })
    );
  }

  #[test]
  fn a_terrain_the_game_does_not_know_is_refused() {
    let table = "1,0,0,A,P,A,R,C,lava";

    assert_eq!(
      parse_definitions(table),
      Err(MapError::UnknownTerrain {
        line: 1,
        terrain: "lava".to_string()
      })
    );
  }

  #[test]
  fn a_row_missing_columns_is_refused() {
    assert_eq!(parse_definitions("1,0,0,A,P"), Err(MapError::MalformedRow { line: 1 }));
  }

  #[test]
  fn a_table_with_no_rows_is_refused() {
    assert_eq!(parse_definitions("# nothing\n"), Err(MapError::NoLocations));
  }
}

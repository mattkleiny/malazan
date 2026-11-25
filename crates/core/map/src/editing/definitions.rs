//! Editing the definitions: renaming locations, moving them between
//! divisions, and adding new ones.

use std::collections::HashSet;

use crate::{
  definitions::parse_definitions,
  locations::{LocationId, Terrain},
  map::{Map, MapError},
};

const HEADER: &str = "red,green,blue,location,province,area,region,continent,terrain";

/// One thing the definitions say about a location.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum LocationField {
  Name,
  Province,
  Area,
  Region,
  Continent,
  Terrain,
}

impl LocationField {
  pub const ALL: [LocationField; 6] = [
    LocationField::Name,
    LocationField::Province,
    LocationField::Area,
    LocationField::Region,
    LocationField::Continent,
    LocationField::Terrain,
  ];

  /// The name the field is chosen by.
  pub fn key(self) -> &'static str {
    match self {
      LocationField::Name => "name",
      LocationField::Province => "province",
      LocationField::Area => "area",
      LocationField::Region => "region",
      LocationField::Continent => "continent",
      LocationField::Terrain => "terrain",
    }
  }

  pub fn from_key(key: &str) -> Option<LocationField> {
    Self::ALL.into_iter().find(|field| field.key() == key)
  }

  /// Where the field sits among a row's six named columns.
  fn column(self) -> usize {
    self as usize
  }
}

/// A row of the definitions: a key colour and its six named columns.
struct Row {
  key_color: [u8; 3],
  fields: [String; 6],
}

impl Row {
  fn line(&self) -> String {
    let [red, green, blue] = self.key_color;

    format!("{red},{green},{blue},{}", self.fields.join(","))
  }
}

impl Map {
  /// The definitions table as the map stands, ready to write to a file.
  pub fn definitions_text(&self) -> String {
    let mut text = String::from(HEADER);

    for row in self.rows() {
      text.push('\n');
      text.push_str(&row.line());
    }

    text.push('\n');
    text
  }

  /// What the definitions say about a location for one field.
  pub fn location_field(&self, location: LocationId, field: LocationField) -> Option<String> {
    let row = self.rows().into_iter().nth(location.index())?;

    Some(row.fields[field.column()].clone())
  }

  /// The map with one field of a location changed. Naming a division that
  /// does not exist yet creates it.
  pub fn with_location_field(&self, location: LocationId, field: LocationField, value: &str) -> Result<Map, MapError> {
    let value = writable(value)?;

    if field == LocationField::Terrain && Terrain::from_key(value).is_none() {
      return Err(MapError::UnknownTerrain {
        line: location.index() + 2,
        terrain: value.to_string(),
      });
    }

    let mut rows = self.rows();
    let row = rows.get_mut(location.index()).ok_or(MapError::UnknownLocation)?;
    row.fields[field.column()] = value.to_string();

    self.with_rows(&rows)
  }

  /// The map with a new location, named `name`, in the same divisions and on
  /// the same terrain as `like`. It covers no ground until painted.
  pub fn with_location_added(&self, name: &str, like: LocationId) -> Result<(Map, LocationId), MapError> {
    let name = writable(name)?;
    let mut rows = self.rows();
    let template = rows.get(like.index()).ok_or(MapError::UnknownLocation)?;

    let mut fields = template.fields.clone();
    fields[LocationField::Name.column()] = name.to_string();

    let taken: HashSet<[u8; 3]> = rows.iter().map(|row| row.key_color).collect();
    let id = LocationId(u16::try_from(rows.len()).map_err(|_| MapError::TooManyLocations)?);

    rows.push(Row {
      key_color: unused_color(&taken, rows.len()),
      fields,
    });

    Ok((self.with_rows(&rows)?, id))
  }

  /// The map without its newest location, which must cover no ground.
  pub fn without_last_location(&self) -> Result<Map, MapError> {
    let mut rows = self.rows();
    let last = LocationId(u16::try_from(rows.len().saturating_sub(1)).map_err(|_| MapError::TooManyLocations)?);
    let row = rows.pop().ok_or(MapError::NoLocations)?;

    if self.raster.cells().contains(&last) {
      return Err(MapError::LocationInUse {
        name: row.fields[LocationField::Name.column()].clone(),
      });
    }

    self.with_rows(&rows)
  }

  fn rows(&self) -> Vec<Row> {
    let name_of = |names: &[String], index: u16| names.get(usize::from(index)).cloned().unwrap_or_default();
    let provinces: Vec<String> = self.provinces.iter().map(|it| it.name.clone()).collect();
    let areas: Vec<String> = self.areas.iter().map(|it| it.name.clone()).collect();
    let regions: Vec<String> = self.regions.iter().map(|it| it.name.clone()).collect();
    let continents: Vec<String> = self.continents.iter().map(|it| it.name.clone()).collect();

    self
      .locations
      .iter()
      .map(|location| {
        let province = self.provinces.get(usize::from(location.province.0));
        let area = province.and_then(|it| self.areas.get(usize::from(it.area.0)));
        let region = area.and_then(|it| self.regions.get(usize::from(it.region.0)));

        Row {
          key_color: location.key_color,
          fields: [
            location.name.clone(),
            name_of(&provinces, location.province.0),
            province.map_or_else(String::new, |it| name_of(&areas, it.area.0)),
            area.map_or_else(String::new, |it| name_of(&regions, it.region.0)),
            region.map_or_else(String::new, |it| name_of(&continents, it.continent.0)),
            location.terrain.key().to_string(),
          ],
        }
      })
      .collect()
  }

  /// The map read again from rows, keeping its painting and heights. Rows
  /// keep their order, so the painting still names the same locations.
  fn with_rows(&self, rows: &[Row]) -> Result<Map, MapError> {
    let text: String = rows.iter().map(|row| row.line() + "\n").collect();
    let definitions = parse_definitions(&text)?;

    if self
      .raster
      .cells()
      .iter()
      .any(|cell| cell.index() >= definitions.locations.len())
    {
      return Err(MapError::MissingLocations);
    }

    Ok(Map::assembled(
      definitions,
      self.raster.clone(),
      self.heights.clone(),
      self.sea_level,
    ))
  }
}

/// The text trimmed, if the definitions can hold it.
fn writable(text: &str) -> Result<&str, MapError> {
  let text = text.trim();

  if text.is_empty() || text.contains([',', '\n', '\r']) {
    return Err(MapError::UnwritableText { text: text.to_string() });
  }

  Ok(text)
}

/// A key colour no location uses yet, the same each time for the same map.
fn unused_color(taken: &HashSet<[u8; 3]>, seed: usize) -> [u8; 3] {
  let mut value = (seed as u32).wrapping_mul(2_654_435_761);

  loop {
    value = value.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);

    let [_, red, green, blue] = value.to_be_bytes();
    let color = [red, green, blue];

    if color != [0, 0, 0] && !taken.contains(&color) {
      return color;
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::{
    heights::Heightfield,
    map::{MapParts, MapPosition},
  };

  const AREN: LocationId = LocationId(0);
  const EHRLITAN: LocationId = LocationId(1);

  /// Two locations in one province, painted half and half.
  fn map() -> Map {
    Map::from_parts(MapParts {
      definitions: "1,0,0,Aren,Coast,Raraku,Seven Cities,Quon Tali,plains\n2,0,0,Ehrlitan,Coast,Raraku,Seven Cities,Quon Tali,desert",
      width: 2,
      height: 1,
      key_colors: &[[1, 0, 0], [2, 0, 0]],
      heights: Heightfield::from_samples(1, 1, vec![0]).unwrap(),
      sea_level: 0.5,
    })
    .unwrap()
  }

  #[test]
  fn the_definitions_read_back_as_they_were_written() {
    let map = map();

    let text = map.definitions_text();

    assert_eq!(
      text,
      "red,green,blue,location,province,area,region,continent,terrain\n1,0,0,Aren,Coast,Raraku,Seven Cities,Quon \
       Tali,plains\n2,0,0,Ehrlitan,Coast,Raraku,Seven Cities,Quon Tali,desert\n"
    );
  }

  #[test]
  fn renaming_a_location_keeps_its_ground() {
    let map = map()
      .with_location_field(AREN, LocationField::Name, "Aren Way")
      .unwrap();

    assert_eq!(map.location(AREN).unwrap().name, "Aren Way");
    assert_eq!(map.location_at(MapPosition { x: 0.5, y: 0.5 }), Some(AREN));
  }

  #[test]
  fn naming_a_new_province_creates_it() {
    let map = map()
      .with_location_field(EHRLITAN, LocationField::Province, "Ehrlitan Hills")
      .unwrap();

    assert_eq!(map.provinces.len(), 2);
    assert_eq!(map.province_of(EHRLITAN).unwrap().name, "Ehrlitan Hills");
    assert_eq!(map.area_of(EHRLITAN).unwrap().name, "Raraku");
  }

  #[test]
  fn moving_a_province_s_only_location_to_another_area_moves_the_province_too() {
    let map = map()
      .with_location_field(EHRLITAN, LocationField::Province, "Hills")
      .unwrap();

    let map = map
      .with_location_field(EHRLITAN, LocationField::Area, "Pan'potsun")
      .unwrap();

    assert_eq!(map.area_of(EHRLITAN).unwrap().name, "Pan'potsun");
  }

  #[test]
  fn splitting_a_province_between_two_areas_is_refused() {
    let result = map().with_location_field(EHRLITAN, LocationField::Area, "Pan'potsun");

    assert!(matches!(result, Err(MapError::SplitDivision { .. })));
  }

  #[test]
  fn a_terrain_the_game_does_not_know_is_refused() {
    let result = map().with_location_field(AREN, LocationField::Terrain, "lava");

    assert!(matches!(result, Err(MapError::UnknownTerrain { .. })));
  }

  #[test]
  fn text_the_definitions_cannot_hold_is_refused() {
    let map = map();

    assert!(matches!(
      map.with_location_field(AREN, LocationField::Name, "A, B"),
      Err(MapError::UnwritableText { .. })
    ));
    assert!(matches!(
      map.with_location_field(AREN, LocationField::Name, "  "),
      Err(MapError::UnwritableText { .. })
    ));
  }

  #[test]
  fn a_new_location_joins_the_divisions_of_the_one_it_is_like() {
    let (map, added) = map().with_location_added("Hissar", EHRLITAN).unwrap();

    assert_eq!(added, LocationId(2));
    assert_eq!(map.location(added).unwrap().name, "Hissar");
    assert_eq!(map.province_of(added).unwrap().name, "Coast");
    assert_eq!(map.location(added).unwrap().terrain, Terrain::Desert);
    assert_ne!(
      map.location(added).unwrap().key_color,
      map.location(AREN).unwrap().key_color
    );
  }

  #[test]
  fn an_unpainted_new_location_can_be_taken_away_again() {
    let (map, _) = map().with_location_added("Hissar", AREN).unwrap();

    let map = map.without_last_location().unwrap();

    assert_eq!(map.locations.len(), 2);
  }

  #[test]
  fn a_painted_location_cannot_be_taken_away() {
    let result = map().without_last_location();

    assert_eq!(
      result.err(),
      Some(MapError::LocationInUse {
        name: "Ehrlitan".to_string()
      })
    );
  }

  #[test]
  fn fields_are_read_by_location() {
    let map = map();

    assert_eq!(
      map.location_field(EHRLITAN, LocationField::Terrain).as_deref(),
      Some("desert")
    );
    assert_eq!(
      map.location_field(AREN, LocationField::Continent).as_deref(),
      Some("Quon Tali")
    );
    assert_eq!(map.location_field(LocationId(5), LocationField::Name), None);
  }
}

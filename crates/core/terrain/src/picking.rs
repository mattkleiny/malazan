//! Picking: finding the point of the map a ray strikes.

use game_map::{LocationId, Map, MapPosition};
use surreal::maths::Vec3;

/// Distance between samples along a ray, measured over the ground (map units).
const STRIDE: f64 = 1.0;

/// The most samples one pick takes before it strides further to keep up.
const MAX_SAMPLES: f64 = 4096.0;

/// How many times a strike is halved to place it.
const REFINEMENTS: usize = 16;

/// A ray through the terrain's space: `x` runs east, `z` south and `y` up, all
/// in map units.
#[derive(Copy, Clone, Debug)]
pub struct Ray {
  pub origin: Vec3,
  pub direction: Vec3, // Unit length.
}

impl Ray {
  fn at(&self, distance: f64) -> Vec3 {
    self.origin + self.direction * distance
  }
}

/// Where a ray strikes the map.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Pick {
  pub position: MapPosition,
  pub location: LocationId,
}

/// Height of the drawn surface at a point (map units): the ground on land,
/// the sea's surface over water. `relief` is the height of the heightfield's
/// full range.
pub fn surface_height(map: &Map, position: MapPosition, relief: f64) -> f64 {
  map.elevation_at(position).max(0.0) * relief
}

/// Finds the first point of the map's surface a ray strikes.
pub fn pick(map: &Map, ray: &Ray, relief: f64) -> Option<Pick> {
  let ceiling = (1.0 - map.sea_level).max(0.0) * relief;
  let (height, fall) = (ray.origin.y, ray.direction.y);

  // The surface lies between the sea and the ceiling, so only that stretch of the
  // ray can strike it.
  let start = if height > ceiling {
    if fall >= 0.0 {
      return None;
    }

    (ceiling - height) / fall
  } else {
    0.0
  };

  let end = if fall < 0.0 {
    -height / fall
  } else if fall > 0.0 {
    (ceiling - height) / fall
  } else {
    start + map.width()
  };
  let end = end.clamp(start, start + map.width() * 2.0);

  let is_under = |distance: f64| {
    let point = ray.at(distance);

    point.y <= surface_height(map, MapPosition { x: point.x, y: point.z }, relief)
  };

  let stride = STRIDE.max((end - start) / MAX_SAMPLES);
  let mut above = start;
  let mut below = None;

  if is_under(start) {
    below = Some(start);
  }

  while below.is_none() && above < end {
    let next = (above + stride).min(end);

    if is_under(next) {
      below = Some(next);
    } else {
      above = next;
    }
  }

  let mut below = below?;

  for _ in 0..REFINEMENTS {
    let middle = (above + below) / 2.0;

    if is_under(middle) {
      below = middle;
    } else {
      above = middle;
    }
  }

  let point = ray.at(below);
  let position = MapPosition { x: point.x, y: point.z };
  let location = map.location_at(position)?;

  Some(Pick { position, location })
}

#[cfg(test)]
mod tests {
  use game_map::{Heightfield, MapParts};

  use super::*;

  const RELIEF: f64 = 10.0;
  const WEST: LocationId = LocationId(0);
  const EAST: LocationId = LocationId(1);

  /// A map eight units wide and four tall, split into a western and an
  /// eastern location, with the given heights stretched over it.
  fn map(heights: Heightfield) -> Map {
    let key_colors: Vec<[u8; 3]> = (0..32)
      .map(|cell| if cell % 8 < 4 { [255, 0, 0] } else { [0, 0, 255] })
      .collect();

    Map::from_parts(MapParts {
      definitions: "255,0,0,West,P,A,R,C,plains\n0,0,255,East,P,A,R,C,plains",
      width: 8,
      height: 4,
      key_colors: &key_colors,
      heights,
      sea_level: 0.5,
    })
    .unwrap()
  }

  /// Open sea everywhere.
  fn sea() -> Map {
    map(Heightfield::from_samples(1, 1, vec![0]).unwrap())
  }

  /// Sea in the west, and a plateau at full height in the east.
  fn plateau() -> Map {
    map(Heightfield::from_samples(8, 1, vec![0, 0, 0, 0, u16::MAX, u16::MAX, u16::MAX, u16::MAX]).unwrap())
  }

  fn ray(origin: [f64; 3], direction: [f64; 3]) -> Ray {
    let direction = Vec3::new(direction[0], direction[1], direction[2]);

    Ray {
      origin: Vec3::new(origin[0], origin[1], origin[2]),
      direction: direction / direction.length(),
    }
  }

  #[test]
  fn a_ray_straight_down_strikes_the_point_beneath_it() {
    let pick = pick(&sea(), &ray([5.5, 20.0, 1.5], [0.0, -1.0, 0.0]), RELIEF).unwrap();

    assert_eq!(pick.location, EAST);
    assert!((pick.position.x - 5.5).abs() < 1e-3 && (pick.position.y - 1.5).abs() < 1e-3);
  }

  #[test]
  fn a_slanted_ray_strikes_where_it_meets_the_sea() {
    let pick = pick(&sea(), &ray([1.0, 3.0, 2.0], [1.0, -1.0, 0.0]), RELIEF).unwrap();

    assert!((pick.position.x - 4.0).abs() < 1e-3);
    assert_eq!(pick.location, EAST);
  }

  #[test]
  fn high_ground_is_struck_before_the_sea_behind_it() {
    let map = plateau();

    // Aimed at the sea-level point x = 7, but the plateau's edge stands in the way.
    let pick = pick(&map, &ray([1.0, 3.0, 2.0], [2.0, -1.0, 0.0]), RELIEF).unwrap();

    assert!(pick.position.x < 5.0, "struck at {}", pick.position.x);
    assert!(surface_height(&map, pick.position, RELIEF) > 0.0);
  }

  #[test]
  fn a_ray_landing_west_of_the_map_strikes_nothing() {
    assert_eq!(pick(&sea(), &ray([-1.5, 5.0, 1.0], [0.0, -1.0, 0.0]), RELIEF), None);
  }

  #[test]
  fn a_ray_pointing_at_the_sky_strikes_nothing() {
    assert_eq!(pick(&sea(), &ray([4.0, 20.0, 2.0], [0.0, 1.0, 0.0]), RELIEF), None);
  }

  #[test]
  fn a_ray_landing_north_of_the_map_strikes_nothing() {
    assert_eq!(pick(&sea(), &ray([4.0, 5.0, -3.0], [0.0, -1.0, 0.0]), RELIEF), None);
  }

  #[test]
  fn the_same_ray_always_strikes_the_same_location() {
    let map = plateau();
    let ray = ray([2.0, 8.0, 0.5], [0.1, -1.0, 0.2]);

    assert_eq!(pick(&map, &ray, RELIEF), pick(&map, &ray, RELIEF));
    assert_eq!(pick(&map, &ray, RELIEF).unwrap().location, WEST);
  }
}

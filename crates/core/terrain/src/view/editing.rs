//! Editing the map in place: painting locations, shaping the ground,
//! changing the definitions, and saving it all back to its map file, labels
//! included.

use game_map::{
  Brush, LocationField, Map, MapContents, MapEdit, MapPosition, Sculpt, Stroke, Terrain, paint_location, sculpt,
};
use godot::{classes::Camera3D, prelude::*, tools::try_save};

use crate::{
  picking::{Ray, pick, surface_height},
  view::{TerrainView, to_location, to_vec3},
};

/// What the view keeps while the map is being edited.
#[derive(Default)]
pub struct EditState {
  stroke: Option<Stroke>,
  edits: Vec<MapEdit>, // Every finished stroke, by its id, for undo and redo.
  is_dirty: bool,      // Whether anything has changed since the map was read or saved.
}

impl EditState {
  /// Notes that the map has changed since it was read or saved.
  pub fn mark_dirty(&mut self) {
    self.is_dirty = true;
  }
}

#[godot_api(secondary)]
impl TerrainView {
  /// Looks at the map from this camera when choosing chunks' detail, as the
  /// editor's camera.
  #[func]
  fn set_view_camera(&mut self, camera: Option<Gd<Camera3D>>) {
    self.view_camera = camera;
  }

  /// The point on the map's surface under a point of the screen, in the view's
  /// own space, or a non-finite vector for none.
  #[func]
  fn pick_point(&self, camera: Gd<Camera3D>, screen_position: Vector2) -> Vector3 {
    let missed = Vector3::new(f32::NAN, f32::NAN, f32::NAN);
    let Some(map) = &self.map else {
      return missed;
    };

    let to_local = self.base().get_global_transform().affine_inverse();
    let ray = Ray {
      origin: to_vec3(to_local * camera.project_ray_origin(screen_position)),
      direction: to_vec3((to_local.basis * camera.project_ray_normal(screen_position)).normalized()),
    };
    let Some(found) = pick(map, &ray, f64::from(self.relief)) else {
      return missed;
    };

    let height = surface_height(map, found.position, f64::from(self.relief));

    Vector3::new(found.position.x as f32, height as f32, found.position.y as f32)
  }

  /// Height of the drawn surface at a point of the map, in the view's own
  /// space.
  #[func]
  fn get_surface_height(&self, x: f64, z: f64) -> f64 {
    self.map.as_ref().map_or(0.0, |map| {
      surface_height(map, MapPosition { x, y: z }, f64::from(self.relief))
    })
  }

  /// The heightfield's value (0..1) at a point of the map, as flattening levels
  /// towards.
  #[func]
  fn get_ground_level(&self, x: f64, z: f64) -> f64 {
    self
      .map
      .as_ref()
      .map_or(0.0, |map| map.heights.height_at(x / map.width(), z / map.height()))
  }

  /// Starts a stroke: every dab until it ends is undone as one.
  #[func]
  fn begin_stroke(&mut self) {
    self.editing.stroke = self.map.as_ref().map(Stroke::begin);
  }

  /// Paints a location over every cell within `radius` (map units) of a point.
  #[func]
  fn paint_location_at(&mut self, x: f64, z: f64, radius: f64, location: i64) {
    let Some(location) = to_location(location) else {
      return;
    };
    let Some(map) = &mut self.map else {
      return;
    };

    let changed = paint_location(map, &brush(x, z, radius, 1.0), location);

    if let Some(stroke) = &mut self.editing.stroke {
      stroke.touch_cells(changed);
    }
    if let (Some(rect), Some(textures)) = (changed, &mut self.textures) {
      textures.refresh_cells(map, rect);
    }
  }

  /// Shapes the ground within `radius` (map units) of a point: raise, lower,
  /// smooth or flatten, towards `level` (0..1) when flattening.
  #[func]
  fn sculpt_at(&mut self, x: f64, z: f64, radius: f64, strength: f64, tool: StringName, level: f64) {
    let shaping = match tool.to_string().as_str() {
      "raise" => Sculpt::Raise,
      "lower" => Sculpt::Lower,
      "smooth" => Sculpt::Smooth,
      "flatten" => Sculpt::Flatten { height: level },
      other => {
        godot_warn!("TerrainView: no sculpting tool '{other}'");
        return;
      }
    };
    let Some(map) = &mut self.map else {
      return;
    };

    let changed = sculpt(map, &brush(x, z, radius, strength), shaping);

    if let Some(stroke) = &mut self.editing.stroke {
      stroke.touch_heights(changed);
    }
    if let (Some(rect), Some(textures)) = (changed, &mut self.textures) {
      textures.refresh_heights(map, rect);
    }
  }

  /// Ends the stroke; the id of its edit for undo and redo, or -1 when it
  /// changed nothing.
  #[func]
  fn end_stroke(&mut self) -> i64 {
    let (Some(stroke), Some(map)) = (self.editing.stroke.take(), &mut self.map) else {
      return -1;
    };
    let Some(edit) = stroke.finish(map) else {
      return -1;
    };

    if edit.after.cells_rect().is_some() {
      map.refresh_adjacency();
    }

    self.editing.edits.push(edit);
    self.editing.is_dirty = true;

    self.editing.edits.len() as i64 - 1
  }

  /// Undoes a finished stroke, or does it again.
  #[func]
  fn apply_edit(&mut self, edit: i64, forwards: bool) {
    let Some(edit) = usize::try_from(edit)
      .ok()
      .and_then(|index| self.editing.edits.get(index))
    else {
      return;
    };
    let Some(map) = &mut self.map else {
      return;
    };

    let patch = if forwards { &edit.after } else { &edit.before };
    map.apply_patch(patch);

    if let Some(textures) = &mut self.textures {
      if let Some(rect) = patch.cells_rect() {
        textures.refresh_cells(map, rect);
      }
      if let Some(rect) = patch.heights_rect() {
        textures.refresh_heights(map, rect);
      }
    }
    if patch.cells_rect().is_some() {
      map.refresh_adjacency();
    }

    self.editing.is_dirty = true;
  }

  /// Every location's name, by id.
  #[func]
  fn get_location_names(&self) -> PackedStringArray {
    self.map.as_ref().map_or_else(PackedStringArray::new, |map| {
      map
        .locations
        .iter()
        .map(|location| GString::from(location.name.as_str()))
        .collect()
    })
  }

  /// The keys of every terrain a location can be.
  #[func]
  fn get_terrains(&self) -> PackedStringArray {
    Terrain::ALL
      .iter()
      .map(|terrain| GString::from(terrain.key()))
      .collect()
  }

  /// What the definitions say about a location: its name, province, area,
  /// region, continent or terrain.
  #[func]
  fn get_location_field(&self, location: i64, field: StringName) -> GString {
    let (Some(map), Some(location), Some(field)) = (
      &self.map,
      to_location(location),
      LocationField::from_key(&field.to_string()),
    ) else {
      return GString::new();
    };

    map
      .location_field(location, field)
      .map_or_else(GString::new, |value| GString::from(value.as_str()))
  }

  /// Changes what the definitions say about a location; why it was refused, or
  /// empty when it was done.
  #[func]
  fn set_location_field(&mut self, location: i64, field: StringName, value: GString) -> GString {
    let Some(field) = LocationField::from_key(&field.to_string()) else {
      return GString::from(format!("no field '{field}'").as_str());
    };
    let (Some(map), Some(location)) = (&self.map, to_location(location)) else {
      return GString::from("no such location");
    };

    match map.with_location_field(location, field, &value.to_string()) {
      Ok(changed) => {
        self.replace_map(changed);
        GString::new()
      }
      Err(error) => GString::from(error.to_string().as_str()),
    }
  }

  /// Adds a location in the same divisions as `like`; its id, or -1 when
  /// refused.
  #[func]
  fn add_location(&mut self, name: GString, like: i64) -> i64 {
    let (Some(map), Some(like)) = (&self.map, to_location(like)) else {
      return -1;
    };

    match map.with_location_added(&name.to_string(), like) {
      Ok((changed, added)) => {
        self.replace_map(changed);
        i64::from(added.0)
      }
      Err(error) => {
        godot_warn!("TerrainView: cannot add '{name}': {error}");
        -1
      }
    }
  }

  /// Takes away the newest location, which must cover no ground; why it was
  /// refused, or empty when it was done.
  #[func]
  fn remove_last_location(&mut self) -> GString {
    let Some(map) = &self.map else {
      return GString::from("no map");
    };

    match map.without_last_location() {
      Ok(changed) => {
        self.replace_map(changed);
        GString::new()
      }
      Err(error) => GString::from(error.to_string().as_str()),
    }
  }

  /// Whether anything has changed since the map was read or saved.
  #[func]
  fn has_unsaved_changes(&self) -> bool {
    self.editing.is_dirty
  }

  /// Writes the map and its labels back to the map file; why it failed, or
  /// empty when it was saved.
  #[func]
  fn save_map(&mut self) -> GString {
    let (Some(map), Some(resource)) = (&self.map, &self.map_resource) else {
      return GString::from("no map file");
    };
    let mut resource = resource.clone();

    resource.bind_mut().contents = Some(MapContents {
      map: map.clone(),
      labels: self.labels.clone(),
    });

    let path = resource.get_path();
    if let Err(error) = try_save(&resource, &path) {
      return GString::from(format!("cannot write {path}: {error}").as_str());
    }

    self.editing.is_dirty = false;
    GString::new()
  }
}

impl TerrainView {
  /// Takes up a map with changed definitions: redraws its borders, ground and
  /// fills, and selects again.
  fn replace_map(&mut self, map: Map) {
    if let Some(textures) = &mut self.textures {
      textures.refresh_borders(&map);
    }

    self.map = Some(map);
    self.editing.is_dirty = true;
    self.upload_terrains();
    self.show_mode();
  }
}

fn brush(x: f64, z: f64, radius: f64, strength: f64) -> Brush {
  Brush {
    centre: MapPosition { x, y: z },
    radius,
    strength,
  }
}

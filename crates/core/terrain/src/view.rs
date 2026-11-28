//! The terrain view: the node that draws the map and picks locations on it.

mod editing;
mod labels;
mod selections;
mod textures;

use game_map::{LocationId, Map, MapContents, MapMode, MapResource, map_mode, map_modes};
use godot::{
  classes::{
    ArrayMesh, Camera3D, INode3D, ImageTexture, MeshInstance3D, Node3D, ShaderMaterial,
    geometry_instance_3d::ShadowCastingSetting,
    mesh::{ArrayType, PrimitiveType},
  },
  prelude::*,
};
use selections::MapSelection;
use surreal::{cvar, maths::Vec3};

use crate::{
  chunks::{ChunkGrid, LEVEL_QUADS, chunk_mesh, level_of_detail},
  picking::{Ray, pick},
  selections::Selections,
  transitions::FillTransition,
  view::textures::{MapTextures, colors_image, highlights_image, terrains_image},
};

cvar!(pub MAP_MODE_FADE: f64, "terrain/map_mode_fade", 0.3, "How long the map takes to fade between map modes (s).");

/// Depth skirts hang below a chunk's rim (map units).
const SKIRT_DEPTH: f32 = 4.0;

/// One piece of the drawn surface.
struct Chunk {
  node: Gd<MeshInstance3D>,
  column: u32,
  row: u32,
  level: usize, // Level of detail it is drawn at.
}

/// Draws the map's surface in chunks, finer near the camera, and finds the
/// location under a point of the screen.
#[derive(GodotClass)]
#[class(init, tool, base = Node3D)]
pub struct TerrainView {
  base: Base<Node3D>,

  /// The map file (.map) to draw.
  #[export]
  map_resource: Option<Gd<MapResource>>,

  /// Height of the tallest peak above the sea (map units).
  #[export(range = (0.0, 500.0))]
  #[init(val = 40.0)]
  relief: f32,

  /// Distance from the camera within which chunks are drawn in full detail (map
  /// units).
  #[export(range = (16.0, 4096.0))]
  #[init(val = 384.0)]
  detail_reach: f32,

  /// The terrain shader's material; the view fills in the map's textures.
  #[export]
  material: Option<Gd<ShaderMaterial>>,

  map: Option<Map>,
  chosen_mode: Option<&'static dyn MapMode>,
  preview_mode: Option<&'static dyn MapMode>, // Shown over the chosen mode while set.
  fills: Option<FillTransition>,
  colors: Option<Gd<ImageTexture>>,
  selections: Selections,
  highlights: Option<Gd<ImageTexture>>,
  grid: Option<ChunkGrid>,
  meshes: Vec<Gd<ArrayMesh>>, // One per level of detail.
  chunks: Vec<Chunk>,
  textures: Option<MapTextures>,
  view_camera: Option<Gd<Camera3D>>, // Looks at the map in place of the viewport's camera, as the editor's does.
  editing: editing::EditState,
  labels: Vec<game_map::MapLabel>,
}

#[godot_api]
impl INode3D for TerrainView {
  fn ready(&mut self) {
    let Some(contents) = self.load_contents() else {
      return;
    };

    self.build(contents.map);
    self.show_labels(contents.labels);
  }

  fn process(&mut self, delta: f64) {
    self.follow_camera();
    self.advance_fade(delta);
  }
}

#[godot_api]
impl TerrainView {
  /// The map's labels were read, or one was changed, added or taken away.
  #[signal]
  fn labels_changed();

  /// The location under a point of the screen, or -1 for none.
  #[func]
  fn pick_location(&self, camera: Gd<Camera3D>, screen_position: Vector2) -> i64 {
    let Some(map) = &self.map else {
      return -1;
    };

    let to_local = self.base().get_global_transform().affine_inverse();
    let origin = to_local * camera.project_ray_origin(screen_position);
    let direction = to_local.basis * camera.project_ray_normal(screen_position);
    let ray = Ray {
      origin: to_vec3(origin),
      direction: to_vec3(direction.normalized()),
    };

    pick(map, &ray, f64::from(self.relief)).map_or(-1, |pick| i64::from(pick.location.0))
  }

  /// The keys of every map mode, in the order they are offered.
  #[func]
  fn get_map_modes(&self) -> PackedStringArray {
    map_modes().iter().map(|mode| GString::from(mode.key())).collect()
  }

  /// The name players see for a map mode.
  #[func]
  fn get_map_mode_label(&self, key: StringName) -> GString {
    map_mode(&key.to_string()).map_or_else(GString::new, |mode| GString::from(mode.label()))
  }

  /// The key of the chosen map mode.
  #[func]
  fn get_map_mode(&self) -> StringName {
    self
      .chosen_mode
      .map_or_else(StringName::default, |mode| StringName::from(mode.key()))
  }

  /// Chooses the map mode the map is coloured by.
  #[func]
  fn set_map_mode(&mut self, key: StringName) {
    let Some(mode) = find_mode(&key) else {
      return;
    };

    self.chosen_mode = Some(mode);
    self.show_mode();
  }

  /// Colours the map afresh, after what the shown map mode draws from has
  /// changed.
  #[func]
  fn refresh_map_mode(&mut self) {
    self.show_mode();
  }

  /// Shows a map mode over the chosen one until the preview is cleared.
  #[func]
  fn preview_map_mode(&mut self, key: StringName) {
    let Some(mode) = find_mode(&key) else {
      return;
    };

    self.preview_mode = Some(mode);
    self.show_mode();
  }

  /// Goes back to the chosen map mode.
  #[func]
  fn clear_map_mode_preview(&mut self) {
    if self.preview_mode.take().is_some() {
      self.show_mode();
    }
  }

  /// Rests the cursor on a location, or on nothing for -1; lights what the map
  /// mode selects there.
  #[func]
  fn hover_location(&mut self, location: i64) {
    let location = to_location(location);
    let (Some(map), Some(mode)) = (&self.map, self.shown_mode()) else {
      return;
    };

    if self.selections.hover(map, mode, location) {
      self.upload_highlights();
    }
  }

  /// Chooses what a location selects in the map mode, or clears the choice for
  /// -1.
  #[func]
  fn select_location(&mut self, location: i64) {
    let location = to_location(location);
    let (Some(map), Some(mode)) = (&self.map, self.shown_mode()) else {
      return;
    };

    if self.selections.select(map, mode, location) {
      self.upload_highlights();
    }
  }

  /// What the cursor rests on, or null.
  #[func]
  fn get_hovered(&self) -> Option<Gd<MapSelection>> {
    self
      .selections
      .hovered
      .selection
      .as_ref()
      .map(MapSelection::from_selection)
  }

  /// What the player chose, or null.
  #[func]
  fn get_selected(&self) -> Option<Gd<MapSelection>> {
    self
      .selections
      .selected
      .selection
      .as_ref()
      .map(MapSelection::from_selection)
  }

  /// Distance from the western edge to the eastern (map units).
  #[func]
  fn get_map_width(&self) -> f64 {
    self.map.as_ref().map_or(0.0, Map::width)
  }

  /// Distance from the northern edge to the southern (map units).
  #[func]
  fn get_map_height(&self) -> f64 {
    self.map.as_ref().map_or(0.0, Map::height)
  }
}

impl TerrainView {
  /// A copy of what the map file holds, for the view to draw and edit.
  fn load_contents(&self) -> Option<MapContents> {
    let Some(resource) = &self.map_resource else {
      godot_warn!("TerrainView: no map file to draw");
      return None;
    };
    let contents = resource.bind().contents.clone();

    if contents.is_none() {
      godot_error!("TerrainView: the map file {} holds no map", resource.get_path());
    }

    contents
  }

  /// Sets up the material and the chunks for a newly loaded map.
  fn build(&mut self, map: Map) {
    let Some(textures) = MapTextures::from_map(&map) else {
      godot_error!("TerrainView: cannot make the map's textures");
      return;
    };

    let grid = ChunkGrid::covering(map.width(), map.height());

    self.set_parameter("heights_tex", &textures.heights.texture.to_variant());
    self.set_parameter("locations_tex", &textures.locations.texture.to_variant());
    self.set_parameter("borders_tex", &textures.borders.texture.to_variant());
    self.set_parameter(
      "map_size",
      &Vector2::new(map.width() as f32, map.height() as f32).to_variant(),
    );
    self.set_parameter("sea_level", &(map.sea_level as f32).to_variant());
    self.set_parameter("relief", &self.relief.to_variant());
    self.set_parameter("skirt_depth", &SKIRT_DEPTH.to_variant());

    self.meshes = LEVEL_QUADS.iter().map(|&quads| self.level_mesh(&grid, quads)).collect();

    for row in 0..grid.rows {
      for column in 0..grid.columns {
        let mut node = MeshInstance3D::new_alloc();

        let [west, north] = grid.origin(column, row);

        node.set_name(&format!("Chunk{column}x{row}"));
        node.set_position(Vector3::new(west as f32, 0.0, north as f32));
        node.set_mesh(&self.meshes[LEVEL_QUADS.len() - 1]);
        node.set_cast_shadows_setting(ShadowCastingSetting::OFF);
        if let Some(material) = &self.material {
          node.set_material_override(material);
        }

        self.base_mut().add_child(&node);
        self.chunks.push(Chunk {
          node,
          column,
          row,
          level: LEVEL_QUADS.len() - 1,
        });
      }
    }

    self.grid = Some(grid);
    self.textures = Some(textures);
    self.map = Some(map);
    self.upload_terrains();
    self.chosen_mode = map_modes().first().copied();
    self.show_mode();
  }

  fn level_mesh(&self, grid: &ChunkGrid, quads: u32) -> Gd<ArrayMesh> {
    let built = chunk_mesh(grid.chunk_width, grid.chunk_depth, quads);
    let vertices: PackedVector3Array = built.vertices.iter().map(|&[x, y, z]| Vector3::new(x, y, z)).collect();
    let indices: PackedInt32Array = built.indices.iter().map(|&index| index as i32).collect();

    let mut arrays = VarArray::new();
    arrays.resize(ArrayType::MAX.ord() as usize, &Variant::nil());
    arrays.set(ArrayType::VERTEX.ord() as usize, &vertices.to_variant());
    arrays.set(ArrayType::INDEX.ord() as usize, &indices.to_variant());

    let mut mesh = ArrayMesh::new_gd();
    mesh.add_surface_from_arrays(PrimitiveType::TRIANGLES, &arrays);
    // Displacement happens in the shader, so the bounds must allow for it.
    mesh.set_custom_aabb(Aabb::new(
      Vector3::new(0.0, -SKIRT_DEPTH, 0.0),
      Vector3::new(
        grid.chunk_width as f32,
        self.relief + SKIRT_DEPTH,
        grid.chunk_depth as f32,
      ),
    ));

    mesh
  }

  /// Moves each chunk to its copy nearest the camera and picks its level of
  /// detail.
  fn follow_camera(&mut self) {
    let Some(grid) = self.grid else {
      return;
    };
    let camera = self
      .view_camera
      .clone()
      .filter(|camera| camera.is_instance_valid())
      .or_else(|| self.base().get_viewport().and_then(|viewport| viewport.get_camera_3d()));
    let Some(camera) = camera else {
      return;
    };

    let to_local = self.base().get_global_transform().affine_inverse();
    let eye = to_local * camera.get_global_position();
    let reach = f64::from(self.detail_reach);
    let (half_width, half_depth) = (grid.chunk_width / 2.0, grid.chunk_depth / 2.0);

    self.set_parameter("terrain_from_world", &to_local.to_variant());

    for chunk in &mut self.chunks {
      let [west, north] = grid.origin(chunk.column, chunk.row);
      let centre = Vector3::new((west + half_width) as f32, 0.0, (north + half_depth) as f32);
      let level = level_of_detail(f64::from(eye.distance_to(centre)), reach);

      if level != chunk.level {
        chunk.node.set_mesh(&self.meshes[level]);
        chunk.level = level;
      }
    }
  }

  /// The previewed map mode, or the chosen one when none is.
  fn shown_mode(&self) -> Option<&'static dyn MapMode> {
    self.preview_mode.or(self.chosen_mode)
  }

  /// Starts fading to the shown mode's fills, and selects again as it sees
  /// things.
  fn show_mode(&mut self) {
    let (Some(map), Some(mode)) = (&self.map, self.shown_mode()) else {
      return;
    };

    let fills = mode.fills(map);
    let selections_changed = self.selections.refresh(map, mode);

    if selections_changed || self.highlights.is_none() {
      self.upload_highlights();
    }

    match &mut self.fills {
      Some(transition) => transition.retarget(fills),
      None => {
        self.fills = Some(FillTransition::settled_on(fills));
        self.upload_fills();
      }
    }
  }

  fn advance_fade(&mut self, delta: f64) {
    let Some(transition) = &mut self.fills else {
      return;
    };

    if transition.is_settled() {
      return;
    }

    transition.advance(delta, MAP_MODE_FADE.get());
    self.upload_fills();
  }

  fn upload_fills(&mut self) {
    let Some(transition) = &self.fills else {
      return;
    };
    let Some(image) = colors_image(&transition.showing()) else {
      godot_error!("TerrainView: cannot make the map mode's colours");
      return;
    };

    if let Some(colors) = &mut self.colors {
      colors.update(&image);
      return;
    }

    let Some(colors) = ImageTexture::create_from_image(&image) else {
      godot_error!("TerrainView: cannot make the map mode's colours");
      return;
    };

    self.set_parameter("colors_tex", &colors.to_variant());
    self.colors = Some(colors);
  }

  /// Tells the shader what kind of ground each location is.
  fn upload_terrains(&mut self) {
    let Some(map) = &self.map else {
      return;
    };
    let Some(terrains) = terrains_image(&map.locations).and_then(|image| ImageTexture::create_from_image(&image))
    else {
      godot_error!("TerrainView: cannot make the terrain lookup");
      return;
    };

    self.set_parameter("terrains_tex", &terrains.to_variant());
  }

  fn upload_highlights(&mut self) {
    let Some(map) = &self.map else {
      return;
    };
    let Some(image) = highlights_image(&self.selections.highlights(map.locations.len())) else {
      godot_error!("TerrainView: cannot make the selection highlights");
      return;
    };

    if let Some(highlights) = &mut self.highlights {
      highlights.update(&image);
      self.show_whether_selected();
      return;
    }

    let Some(highlights) = ImageTexture::create_from_image(&image) else {
      godot_error!("TerrainView: cannot make the selection highlights");
      return;
    };

    self.set_parameter("highlights_tex", &highlights.to_variant());
    self.highlights = Some(highlights);
    self.show_whether_selected();
  }

  fn show_whether_selected(&mut self) {
    let has_selection = self.selections.selected.selection.is_some();

    self.set_parameter("has_selection", &has_selection.to_variant());
  }

  fn set_parameter(&mut self, name: &str, value: &Variant) {
    if let Some(material) = &mut self.material {
      material.set_shader_parameter(name, value);
    }
  }
}

fn find_mode(key: &StringName) -> Option<&'static dyn MapMode> {
  let mode = map_mode(&key.to_string());

  if mode.is_none() {
    godot_warn!("TerrainView: no map mode '{key}'");
  }

  mode
}

/// A location id from Godot, where -1 or anything out of range means none.
fn to_location(location: i64) -> Option<LocationId> {
  u16::try_from(location).ok().map(LocationId)
}

fn to_vec3(vector: Vector3) -> Vec3 {
  Vec3::new(f64::from(vector.x), f64::from(vector.y), f64::from(vector.z))
}

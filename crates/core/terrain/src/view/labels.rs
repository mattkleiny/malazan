//! Map labels as Godot sees them, and editing them in place.

use game_map::MapPosition;
use godot::prelude::*;

use crate::view::TerrainView;

/// A name lettered across the map, where the designer placed it.
#[derive(GodotClass)]
#[class(init, base = RefCounted)]
pub struct MapLabel {
  #[var]
  text: GString,

  /// Where the middle of the lettering sits, in map units: `x` east from the
  /// western edge, `y` south from the northern edge.
  #[var]
  position: Vector2,

  /// Height of the lettering (map units).
  #[var]
  #[init(val = 20.0)]
  size: f32,

  /// How far the lettering is turned anticlockwise from reading west to east
  /// (degrees).
  #[var]
  angle: f32,

  /// Whether it names a sea or a lake, and is inked as water.
  #[var]
  is_water: bool,
}

impl MapLabel {
  fn from_label(label: &game_map::MapLabel) -> Gd<Self> {
    Gd::from_object(Self {
      text: GString::from(label.text.as_str()),
      position: Vector2::new(label.position.x as f32, label.position.y as f32),
      size: label.size as f32,
      angle: label.angle as f32,
      is_water: label.is_water,
    })
  }

  fn to_label(&self) -> game_map::MapLabel {
    game_map::MapLabel {
      text: self.text.to_string(),
      position: MapPosition {
        x: f64::from(self.position.x),
        y: f64::from(self.position.y),
      },
      size: f64::from(self.size),
      angle: f64::from(self.angle),
      is_water: self.is_water,
    }
  }
}

#[godot_api(secondary)]
impl TerrainView {
  /// Every label on the map, in the order they are kept.
  #[func]
  fn get_map_labels(&self) -> Array<Gd<MapLabel>> {
    self.labels.iter().map(MapLabel::from_label).collect()
  }

  /// One label, or null for none.
  #[func]
  fn get_map_label(&self, index: i64) -> Option<Gd<MapLabel>> {
    let label = usize::try_from(index).ok().and_then(|index| self.labels.get(index))?;

    Some(MapLabel::from_label(label))
  }

  /// Changes a label; why it was refused, or empty when it was done.
  #[func]
  fn set_map_label(&mut self, index: i64, label: Gd<MapLabel>) -> GString {
    let label = label.bind().to_label();
    if let Err(error) = label.check_writable() {
      return GString::from(error.to_string().as_str());
    }
    let Some(kept) = usize::try_from(index).ok().and_then(|index| self.labels.get_mut(index)) else {
      return GString::from("no such label");
    };

    *kept = label;
    self.labels_changed();
    GString::new()
  }

  /// Puts a label among the others at `index`, or at the end for -1; why it
  /// was refused, or empty when it was done.
  #[func]
  fn insert_map_label(&mut self, index: i64, label: Gd<MapLabel>) -> GString {
    let label = label.bind().to_label();
    if let Err(error) = label.check_writable() {
      return GString::from(error.to_string().as_str());
    }
    let index = usize::try_from(index).unwrap_or(self.labels.len());
    if index > self.labels.len() {
      return GString::from("no such place among the labels");
    }

    self.labels.insert(index, label);
    self.labels_changed();
    GString::new()
  }

  /// Takes a label off the map.
  #[func]
  fn remove_map_label(&mut self, index: i64) {
    let Some(index) = usize::try_from(index).ok().filter(|&index| index < self.labels.len()) else {
      return;
    };

    self.labels.remove(index);
    self.labels_changed();
  }
}

impl TerrainView {
  /// Takes up the labels read from the map file.
  pub fn show_labels(&mut self, labels: Vec<game_map::MapLabel>) {
    self.labels = labels;
    self.signals().labels_changed().emit();
  }

  fn labels_changed(&mut self) {
    self.editing.mark_dirty();
    self.signals().labels_changed().emit();
  }
}

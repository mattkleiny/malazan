//! Selections as Godot sees them.

use game_map::Selection;
use godot::prelude::*;

/// A thing chosen on the map, described for the player.
#[derive(GodotClass)]
#[class(no_init, base = RefCounted)]
pub struct MapSelection {
  /// What sort of thing it is, such as region.
  #[var(no_set)]
  kind: StringName,

  #[var(no_set)]
  title: GString,

  /// Lines describing it to the player.
  #[var(no_set)]
  details: PackedStringArray,
}

impl MapSelection {
  pub fn from_selection(selection: &Selection) -> Gd<Self> {
    Gd::from_object(Self {
      kind: StringName::from(selection.kind),
      title: GString::from(selection.title.as_str()),
      details: selection
        .details
        .iter()
        .map(|line| GString::from(line.as_str()))
        .collect(),
    })
  }
}

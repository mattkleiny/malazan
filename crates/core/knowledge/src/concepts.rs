//! Concepts: the ideas of the game a player can look up.

use godot::prelude::*;

/// One idea of the game the player can look up: a name, and a description
/// that links on to other knowledge.
#[derive(GodotClass)]
#[class(init, tool, base = Resource)]
pub struct KnowledgeConcept {
  base: Base<Resource>,

  /// What knowledge links name this concept by.
  #[export]
  pub id: StringName,

  /// The name shown to the player.
  #[export]
  pub title: GString,

  /// The description, written in knowledge markup.
  #[export(multiline)]
  pub body: GString,
}

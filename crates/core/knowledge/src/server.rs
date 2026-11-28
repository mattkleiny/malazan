//! The knowledge base: every concept by its id, and the markup that links them.

use std::{cell::OnceCell, collections::HashMap};

use godot::prelude::*;
use surreal::{
  cvar,
  godot::{load_all_from, singleton},
  maths::Color32,
};

use crate::{
  concepts::KnowledgeConcept,
  links::{Link, LinkStyle, Piece, pieces},
};

cvar!(pub KNOWLEDGE_LINK_COLOR: Color32, "ui/knowledge/link_color", Color32::rgb(0.95, 0.85, 0.55), "Colour of a knowledge link that leads somewhere.");
cvar!(pub KNOWLEDGE_BROKEN_LINK_COLOR: Color32, "ui/knowledge/broken_link_color", Color32::rgb(0.9, 0.35, 0.3), "Colour of a knowledge link naming nothing known.");
cvar!(pub CONCEPTS_DIRECTORY: String, "ui/knowledge/concepts_directory", "res://knowledge/concepts", "Directory where knowledge concepts are stored.");

/// The link kind naming a concept.
const CONCEPT_KIND: &str = "concept";

/// The game's knowledge: every concept by its id, and the markup that links
/// them, turned into BBCode for labels to show.
#[singleton("KnowledgeBase")]
#[derive(GodotClass)]
#[class(init, tool, base = Object)]
pub struct KnowledgeBase {
  base: Base<Object>,
  concepts: OnceCell<HashMap<String, Gd<KnowledgeConcept>>>, // Loaded on first use.
}

#[godot_api]
impl KnowledgeBase {
  /// The concept with this id, or null when there is none.
  #[func]
  fn get_concept(&self, id: StringName) -> Option<Gd<KnowledgeConcept>> {
    self.concepts().get(&id.to_string()).cloned()
  }

  /// What a link written as `kind:id` leads to, or null when it names nothing
  /// known.
  #[func]
  fn resolve(&self, link: GString) -> Option<Gd<KnowledgeConcept>> {
    let link = link.to_string();
    let (kind, id) = link.split_once(':')?;

    self.concept_for(Link { kind, id, shown: None })
  }

  /// Turns knowledge markup into BBCode, each link becoming coloured text a
  /// label reports the cursor on.
  #[func]
  fn format(&self, markup: GString) -> GString {
    let style = LinkStyle {
      color: hex(KNOWLEDGE_LINK_COLOR.get()),
      broken_color: hex(KNOWLEDGE_BROKEN_LINK_COLOR.get()),
    };

    let markup = markup.to_string();
    let mut bbcode = String::with_capacity(markup.len());
    for piece in pieces(&markup) {
      match piece {
        Piece::Text(text) => bbcode.push_str(text),
        Piece::Link(link) => {
          let title = self.concept_for(link).map(|concept| concept.bind().title.to_string());
          if title.is_none() {
            godot_warn!("Knowledge link '{}' names nothing known", link.reference());
          }
          bbcode.push_str(&link.to_bbcode(title.as_deref(), &style));
        }
      }
    }

    GString::from(bbcode.as_str())
  }
}

impl KnowledgeBase {
  /// The concept with this id.
  pub fn concept(&self, id: &str) -> Option<Gd<KnowledgeConcept>> {
    self.concepts().get(id).cloned()
  }

  fn concept_for(&self, link: Link) -> Option<Gd<KnowledgeConcept>> {
    if link.kind != CONCEPT_KIND {
      return None;
    }
    self.concept(link.id)
  }

  fn concepts(&self) -> &HashMap<String, Gd<KnowledgeConcept>> {
    self.concepts.get_or_init(load_concepts)
  }
}

/// Loads every concept saved in the concepts directory, by its id.
fn load_concepts() -> HashMap<String, Gd<KnowledgeConcept>> {
  let mut concepts: HashMap<String, Gd<KnowledgeConcept>> = HashMap::new();

  for concept in load_all_from::<KnowledgeConcept>(&[&CONCEPTS_DIRECTORY.get()]) {
    let id = concept.bind().id.to_string();

    concepts.insert(id, concept);
  }

  concepts
}

/// A colour as BBCode writes it: hex, without the `#` or alpha.
fn hex(color: Color32) -> String {
  let channel = |value: f64| (value.clamp(0.0, 1.0) * 255.0).round() as u8;

  format!(
    "{:02x}{:02x}{:02x}",
    channel(color.r),
    channel(color.g),
    channel(color.b)
  )
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn colours_are_written_as_six_hex_digits() {
    let written = hex(Color32::rgb(1.0, 0.5, 0.0));

    assert_eq!(written, "ff8000");
  }
}

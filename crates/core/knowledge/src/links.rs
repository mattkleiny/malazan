//! Knowledge links: mentions of knowledge inside text, and the markup they are
//! written in.

/// A mention of a piece of knowledge, written `[kind:id]`, or
/// `[kind:id|shown text]` to word it differently.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Link<'a> {
  pub kind: &'a str, // What sort of knowledge the id names, such as concept.
  pub id: &'a str,
  pub shown: Option<&'a str>, // Shown in place of the knowledge's own name.
}

impl Link<'_> {
  /// The link as a label reports it under the cursor: `kind:id`.
  pub fn reference(self) -> String {
    format!("{}:{}", self.kind, self.id)
  }

  /// The link as BBCode, given the name of what it leads to, or none when it
  /// names nothing known.
  pub fn to_bbcode(self, title: Option<&str>, style: &LinkStyle) -> String {
    let Some(title) = title else {
      let shown = self.shown.map_or_else(|| self.reference(), str::to_owned);
      return format!("[color=#{}]{shown}[/color]", style.broken_color);
    };

    format!(
      "[url={}][color=#{}]{}[/color][/url]",
      self.reference(),
      style.color,
      self.shown.unwrap_or(title)
    )
  }
}

/// One run of knowledge markup.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Piece<'a> {
  Text(&'a str),
  Link(Link<'a>),
}

/// How links are coloured once written as BBCode, as hex without the `#`.
pub struct LinkStyle {
  pub color: String,        // A link that leads somewhere.
  pub broken_color: String, // A link naming nothing known.
}

/// Splits knowledge markup into text and links. Anything else in brackets,
/// such as BBCode, stays text.
pub fn pieces(markup: &str) -> Vec<Piece<'_>> {
  let mut pieces = Vec::new();
  let mut text_start = 0;
  let mut search = 0;

  while let Some(offset) = markup[search..].find('[') {
    let open = search + offset;
    let Some((link, length)) = parse_link(&markup[open..]) else {
      search = open + 1;
      continue;
    };

    if open > text_start {
      pieces.push(Piece::Text(&markup[text_start..open]));
    }
    pieces.push(Piece::Link(link));
    text_start = open + length;
    search = text_start;
  }

  if text_start < markup.len() {
    pieces.push(Piece::Text(&markup[text_start..]));
  }
  pieces
}

/// Reads the link `text` opens with, and how many bytes it spans.
fn parse_link(text: &str) -> Option<(Link<'_>, usize)> {
  let close = text.find(']')?;
  let inner = text.get(1..close)?;
  let (reference, shown) = match inner.split_once('|') {
    Some((reference, shown)) => (reference, Some(shown).filter(|shown| !shown.is_empty())),
    None => (inner, None),
  };

  let (kind, id) = reference.split_once(':')?;
  if !is_word(kind) || !is_word(id) {
    return None;
  }

  Some((Link { kind, id, shown }, close + 1))
}

fn is_word(text: &str) -> bool {
  !text.is_empty() && text.chars().all(|c| c.is_alphanumeric() || c == '_')
}

#[cfg(test)]
mod tests {
  use super::*;

  fn concept(id: &str) -> Link<'_> {
    Link {
      kind: "concept",
      id,
      shown: None,
    }
  }

  fn style() -> LinkStyle {
    LinkStyle {
      color: "aaaaaa".into(),
      broken_color: "ff0000".into(),
    }
  }

  #[test]
  fn markup_splits_into_text_and_links() {
    let pieces = pieces("A group of [concept:province] and more.");

    assert_eq!(pieces, [
      Piece::Text("A group of "),
      Piece::Link(concept("province")),
      Piece::Text(" and more.")
    ]);
  }

  #[test]
  fn a_link_can_be_worded_differently() {
    let pieces = pieces("[concept:location|locations]");

    assert_eq!(pieces, [Piece::Link(Link {
      shown: Some("locations"),
      ..concept("location")
    })]);
  }

  #[test]
  fn empty_wording_falls_back_to_the_name() {
    let pieces = pieces("[concept:location|]");

    assert_eq!(pieces, [Piece::Link(concept("location"))]);
  }

  #[test]
  fn bbcode_is_left_as_text() {
    let pieces = pieces("[b]bold[/b] [color=#fff]x[/color] [url=a:b]y[/url]");

    assert_eq!(pieces, [Piece::Text(
      "[b]bold[/b] [color=#fff]x[/color] [url=a:b]y[/url]"
    )]);
  }

  #[test]
  fn a_link_inside_bbcode_is_still_found() {
    let pieces = pieces("[b][concept:map][/b]");

    assert_eq!(pieces, [
      Piece::Text("[b]"),
      Piece::Link(concept("map")),
      Piece::Text("[/b]")
    ]);
  }

  #[test]
  fn an_unclosed_bracket_is_text() {
    let pieces = pieces("see [concept:map");

    assert_eq!(pieces, [Piece::Text("see [concept:map")]);
  }

  #[test]
  fn a_known_link_becomes_a_coloured_url() {
    let bbcode = concept("map_mode").to_bbcode(Some("Map Mode"), &style());

    assert_eq!(bbcode, "[url=concept:map_mode][color=#aaaaaa]Map Mode[/color][/url]");
  }

  #[test]
  fn a_known_link_keeps_its_own_wording() {
    let link = Link {
      shown: Some("provinces"),
      ..concept("province")
    };

    let bbcode = link.to_bbcode(Some("Province"), &style());

    assert_eq!(bbcode, "[url=concept:province][color=#aaaaaa]provinces[/color][/url]");
  }

  #[test]
  fn a_broken_link_is_coloured_text_without_a_url() {
    let bbcode = concept("nowhere").to_bbcode(None, &style());

    assert_eq!(bbcode, "[color=#ff0000]concept:nowhere[/color]");
  }
}

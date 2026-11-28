//! Map labels: names lettered by hand across the map, and the table they are
//! kept in.

use std::fmt;

use crate::map::MapPosition;

const HEADER: &str = "text,x,y,size,angle,kind";
const COLUMNS: usize = 6;

/// A name lettered across the map, where the designer placed it.
#[derive(Clone, Debug, PartialEq)]
pub struct MapLabel {
  pub text: String,
  pub position: MapPosition, // Where the middle of the lettering sits.
  pub size: f64,             // Height of the lettering (map units).
  pub angle: f64,            // How far the lettering is turned anticlockwise from reading west to east (degrees).
  pub is_water: bool,        // Whether it names a sea or a lake, and is inked as water.
}

/// Why the labels table could not be read, or a label could not be kept in it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LabelError {
  MalformedRow { line: usize },
  UnknownKind { line: usize, kind: String },
  UnwritableText { text: String },
}

impl fmt::Display for LabelError {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Self::MalformedRow { line } => write!(f, "labels line {line}: expected text, x, y, size, angle and kind"),
      Self::UnknownKind { line, kind } => write!(f, "labels line {line}: unknown kind '{kind}', not land or water"),
      Self::UnwritableText { text } => write!(
        f,
        "'{text}' cannot be written to the labels: it is empty or holds a comma or line break"
      ),
    }
  }
}

impl MapLabel {
  /// Checks the label can be written to the table and read back the same.
  pub fn check_writable(&self) -> Result<(), LabelError> {
    let text = self.text.trim();

    if text.is_empty() || text != self.text || text.contains([',', '\n', '\r']) {
      return Err(LabelError::UnwritableText {
        text: self.text.clone(),
      });
    }

    Ok(())
  }

  fn line(&self) -> String {
    let kind = if self.is_water { "water" } else { "land" };

    format!(
      "{},{},{},{},{},{kind}",
      self.text,
      rounded(self.position.x),
      rounded(self.position.y),
      rounded(self.size),
      rounded(self.angle)
    )
  }
}

/// Reads the labels table: one comma-separated row per label, giving
/// `text,x,y,size,angle,kind`. Blank lines, `#` comments and the header row
/// are skipped.
pub fn parse_labels(text: &str) -> Result<Vec<MapLabel>, LabelError> {
  let mut labels = Vec::new();

  for (index, row) in text.lines().enumerate() {
    let line = index + 1;
    let row = row.trim();

    if row.is_empty() || row.starts_with('#') || row == HEADER {
      continue;
    }

    let fields: Vec<&str> = row.split(',').map(str::trim).collect();
    if fields.len() != COLUMNS || fields[0].is_empty() {
      return Err(LabelError::MalformedRow { line });
    }

    let number = |field: &str| {
      field
        .parse::<f64>()
        .ok()
        .filter(|value| value.is_finite())
        .ok_or(LabelError::MalformedRow { line })
    };
    let is_water = match fields[5] {
      "land" => false,
      "water" => true,
      kind => {
        return Err(LabelError::UnknownKind {
          line,
          kind: kind.to_string(),
        });
      }
    };

    labels.push(MapLabel {
      text: fields[0].to_string(),
      position: MapPosition {
        x: number(fields[1])?,
        y: number(fields[2])?,
      },
      size: number(fields[3])?,
      angle: number(fields[4])?,
      is_water,
    });
  }

  Ok(labels)
}

/// The labels table, ready to write to a file.
pub fn labels_text(labels: &[MapLabel]) -> String {
  let mut text = String::from(HEADER);

  for label in labels {
    text.push('\n');
    text.push_str(&label.line());
  }

  text.push('\n');
  text
}

/// A number to a tenth, which is finer than lettering is placed by eye.
fn rounded(value: f64) -> f64 {
  (value * 10.0).round() / 10.0
}

#[cfg(test)]
mod tests {
  use super::*;

  fn label(text: &str) -> MapLabel {
    MapLabel {
      text: text.to_string(),
      position: MapPosition { x: 120.5, y: 64.0 },
      size: 18.0,
      angle: -12.5,
      is_water: false,
    }
  }

  #[test]
  fn written_labels_read_back_the_same() {
    let labels = vec![label("Blackreed Fens"), MapLabel {
      is_water: true,
      ..label("Ocean of Sighs")
    }];

    let read = parse_labels(&labels_text(&labels)).unwrap();

    assert_eq!(read, labels);
  }

  #[test]
  fn comments_blank_lines_and_the_header_are_skipped() {
    let text = "text,x,y,size,angle,kind\n\n# the north\nThe Rimeshelf,768.5,204.5,34,0,land\n";

    let labels = parse_labels(text).unwrap();

    assert_eq!(labels.len(), 1);
    assert_eq!(labels[0].text, "The Rimeshelf");
    assert_eq!(labels[0].position, MapPosition { x: 768.5, y: 204.5 });
  }

  #[test]
  fn a_row_missing_a_column_is_reported_with_its_line() {
    let text = "Veilwood,576.5,1476.5,27.5,0,land\nGallowmere,144.5,1356.5,13,land";

    assert_eq!(parse_labels(text), Err(LabelError::MalformedRow { line: 2 }));
  }

  #[test]
  fn a_kind_other_than_land_or_water_is_reported() {
    let result = parse_labels("Lake Ysul,768.5,1044.5,34,0,lake");

    assert_eq!(
      result,
      Err(LabelError::UnknownKind {
        line: 1,
        kind: "lake".to_string()
      })
    );
  }

  #[test]
  fn a_label_holding_a_comma_cannot_be_written() {
    let result = label("Fens, Black").check_writable();

    assert_eq!(
      result,
      Err(LabelError::UnwritableText {
        text: "Fens, Black".to_string()
      })
    );
  }
}

//! Map mode transitions: fading the map's fills from one mode to the next.

use game_map::Fill;

/// A fade from the fills the map showed to the fills it should show.
#[derive(Clone, Debug, PartialEq)]
pub struct FillTransition {
  from: Vec<Fill>,
  to: Vec<Fill>,
  progress: f64, // 0..1
}

impl FillTransition {
  /// A transition already at rest on `fills`.
  pub fn settled_on(fills: Vec<Fill>) -> Self {
    Self {
      from: fills.clone(),
      to: fills,
      progress: 1.0,
    }
  }

  /// Starts fading towards `fills` from whatever is showing now, so a mode
  /// chosen mid-fade never jumps.
  pub fn retarget(&mut self, fills: Vec<Fill>) {
    self.from = self.showing();
    self.to = fills;
    self.progress = 0.0;
  }

  /// Moves the fade on by `delta` of a fade lasting `duration` (s).
  pub fn advance(&mut self, delta: f64, duration: f64) {
    if duration <= 0.0 {
      self.progress = 1.0;
    } else {
      self.progress = (self.progress + delta.max(0.0) / duration).min(1.0);
    }
  }

  pub fn is_settled(&self) -> bool {
    self.progress >= 1.0
  }

  /// The fills to draw at this point of the fade.
  pub fn showing(&self) -> Vec<Fill> {
    if self.is_settled() {
      return self.to.clone();
    }

    // Eased, so the fade starts and ends gently.
    let amount = self.progress * self.progress * (3.0 - 2.0 * self.progress);

    self
      .to
      .iter()
      .enumerate()
      .map(|(index, &to)| blend(self.from.get(index).copied().unwrap_or(Fill::CLEAR), to, amount))
      .collect()
  }
}

/// A fill part of the way from one to another. A clear fill has no colour of
/// its own, so fading to or from it keeps the other's colour.
fn blend(from: Fill, to: Fill, amount: f64) -> Fill {
  let (from_weight, to_weight) = (
    f64::from(from.strength) * (1.0 - amount),
    f64::from(to.strength) * amount,
  );
  let strength = from_weight + to_weight;

  if strength <= 0.0 {
    return Fill::CLEAR;
  }

  let channel = |index: usize| {
    let mixed = (f64::from(from.color[index]) * from_weight + f64::from(to.color[index]) * to_weight) / strength;

    mixed.round() as u8
  };

  Fill {
    color: [channel(0), channel(1), channel(2)],
    strength: strength.round() as u8,
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  const RED: Fill = Fill {
    color: [200, 0, 0],
    strength: 255,
  };
  const BLUE: Fill = Fill {
    color: [0, 0, 200],
    strength: 255,
  };

  #[test]
  fn a_fade_starts_on_the_old_fills_and_ends_on_the_new() {
    let mut transition = FillTransition::settled_on(vec![RED]);

    transition.retarget(vec![BLUE]);
    assert_eq!(transition.showing(), [RED]);

    transition.advance(1.0, 1.0);
    assert_eq!(transition.showing(), [BLUE]);
    assert!(transition.is_settled());
  }

  #[test]
  fn halfway_through_a_fade_shows_an_even_mix() {
    let mut transition = FillTransition::settled_on(vec![RED]);

    transition.retarget(vec![BLUE]);
    transition.advance(0.5, 1.0);

    assert_eq!(transition.showing(), [Fill {
      color: [100, 0, 100],
      strength: 255
    }]);
  }

  #[test]
  fn a_mode_chosen_mid_fade_carries_on_from_what_is_showing() {
    let mut transition = FillTransition::settled_on(vec![RED]);
    transition.retarget(vec![BLUE]);
    transition.advance(0.5, 1.0);
    let showing = transition.showing();

    transition.retarget(vec![RED]);

    assert_eq!(transition.showing(), showing);
    assert!(!transition.is_settled());
  }

  #[test]
  fn fading_to_clear_keeps_the_colour_and_loses_strength() {
    let mut transition = FillTransition::settled_on(vec![RED]);

    transition.retarget(vec![Fill::CLEAR]);
    transition.advance(0.5, 1.0);

    assert_eq!(transition.showing(), [Fill {
      color: [200, 0, 0],
      strength: 128
    }]);
  }

  #[test]
  fn a_fade_with_no_duration_settles_at_once() {
    let mut transition = FillTransition::settled_on(vec![RED]);

    transition.retarget(vec![BLUE]);
    transition.advance(0.0, 0.0);

    assert_eq!(transition.showing(), [BLUE]);
  }

  #[test]
  fn several_steps_add_up_to_one_fade() {
    let mut transition = FillTransition::settled_on(vec![RED]);
    transition.retarget(vec![BLUE]);

    transition.advance(0.25, 1.0);
    assert!(!transition.is_settled());
    transition.advance(0.75, 1.0);

    assert!(transition.is_settled());
  }
}

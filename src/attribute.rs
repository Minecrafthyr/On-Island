use std::fmt::Display;

use strum::IntoEnumIterator;
use strum_macros::{EnumCount, EnumIter, EnumString, IntoStaticStr};
use time::Duration;

use crate::{
  units::{DurationDisplay, hours, seconds},
  utils::{DetailedDisplay, NameAndDesc},
};

#[derive(Clone, Copy, PartialEq)]
pub enum AttributeValue {
  Dur(Duration),
  Float(f64),
}

impl AttributeValue {
  pub fn positive_signed(&self) -> String {
    if match self {
    Self::Dur(duration) => duration.is_positive(),
    Self::Float(float) => float.is_sign_positive(),
    } {
      format!("+{self}")
    } else {
      self.to_string()
    }
  }
}
impl Display for AttributeValue {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
    Self::Dur(duration) => {
      write!(f, "{}", DurationDisplay(*duration))
    }
    Self::Float(float) => write!(f, "{:2}%", float * 100.0),
    }
  }
}
impl Display for DetailedDisplay<AttributeValue> {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    use AttributeValue::*;
    match self.0 {
    Dur(duration) => {
      write!(f, "{}", DurationDisplay(duration))
    }
    Float(float) => write!(f, "{:3}%", float * 100.0),
    }
  }
}

impl From<Duration> for AttributeValue {
  fn from(d: Duration) -> Self { Self::Dur(d) }
}

impl From<f64> for AttributeValue {
  fn from(f: f64) -> Self { Self::Float(f) }
}

#[derive(EnumIter, EnumCount, Clone, Copy, PartialEq, Eq, EnumString, IntoStaticStr)]
#[strum(serialize_all = "snake_case")]
pub enum Attribute {
  Health,
  Energy,
  Water,
}
pub use Attribute::*;

impl NameAndDesc for Attribute {
  const PREFIX: &str = "attribute";

  fn get_id(&self) -> &str { self.into() }
}

#[derive(Clone)]
pub struct Attributes {
  pub health: f64,
  pub energy: Duration,
  pub water: Duration,
}
impl Display for Attributes {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    for attr in Attribute::iter() {
      f.write_str(&t!("game.stats", name = attr.name(), value = self.get(attr)))?;
    }
    Ok(())
  }
}

// 通用的属性修改器
#[derive(Clone, Copy)]
pub struct AttributeModifier {
  pub attribute: Attribute,
  pub value: AttributeValue,
}

impl Display for AttributeModifier {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}{}", self.attribute.name(), self.value)
  }
}
impl AttributeModifier {
  pub fn new(attribute: Attribute, value: impl Into<AttributeValue>) -> Self {
    Self { attribute, value: value.into() }
  }

  pub fn apply(&self, attributes: &mut Attributes) {
    match (self.attribute, self.value) {
    (Attribute::Health, AttributeValue::Float(v)) => {
      attributes.health = v;
    }
    (Attribute::Energy, AttributeValue::Dur(v)) => {
      attributes.energy = v;
    }
    (Attribute::Energy, AttributeValue::Float(v)) => {
      attributes.energy = seconds(v.round() as i64);
    }
    (Attribute::Water, AttributeValue::Dur(v)) => {
      attributes.water = v;
    }
    (Attribute::Water, AttributeValue::Float(v)) => {
      attributes.water = seconds(v.round() as i64);
    }
    _ => (),
    }
  }
}
const impl From<(Attribute, AttributeValue)> for AttributeModifier {
  fn from(value: (Attribute, AttributeValue)) -> Self {
    Self { attribute: value.0, value: value.1 }
  }
}

impl Default for Attributes {
  fn default() -> Self { Self::new() }
}

impl Attributes {
  pub fn iter(&self) -> impl Iterator<Item = AttributeValue> {
    Attribute::iter().map(|attr| self.get(attr))
  }

  pub fn new() -> Self { Self { health: 1.0, energy: hours(72), water: hours(72) } }

  pub fn get(&self, attribute: Attribute) -> AttributeValue {
    match attribute {
    Attribute::Health => AttributeValue::Float(self.health),
    Attribute::Energy => AttributeValue::Dur(self.energy),
    Attribute::Water => AttributeValue::Dur(self.water),
    }
  }

  pub fn apply_modifier(&mut self, modifier: &AttributeModifier) { modifier.apply(self); }
}

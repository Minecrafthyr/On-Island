use std::fmt::{Display, Write};

use strum_macros::{EnumCount, EnumIter, EnumString, IntoStaticStr};
use time::Duration;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AttributeValue {
  Dur(Duration),
  Float(f64),
}

impl AttributeValue {
  pub fn positive_signed(&self) -> String {
    let mut r = if match self {
    Self::Dur(duration) => duration.is_positive(),
    Self::Float(float) => float.is_sign_positive(),
    } {
      String::from("+")
    } else {
      String::new()
    };
    write!(r, "{self}").unwrap();
    r
  }
}
impl Display for AttributeValue {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
    Self::Dur(duration) => write!(f, "{duration}"),
    Self::Float(float) => write!(f, "{float}"),
    }
  }
}

impl From<Duration> for AttributeValue {
  fn from(d: Duration) -> Self { Self::Dur(d) }
}

impl From<f64> for AttributeValue {
  fn from(f: f64) -> Self { Self::Float(f) }
}

#[derive(Debug, EnumIter, EnumCount, Clone, Copy, PartialEq, Eq, EnumString, IntoStaticStr)]
#[strum(serialize_all = "snake_case")]
pub enum Attribute {
  Health,
  Energy,
  Water,
}
pub use Attribute::*;

use crate::{
  units::{hours, seconds},
  utils::NameAndDesc,
};

impl NameAndDesc for Attribute {
  const PREFIX: &str = "attribute";

  fn get_id(&self) -> &str { self.into() }
}

#[derive(Debug, Clone)]
pub struct Attributes {
  pub health: f64,
  pub energy: Duration,
  pub water: Duration,
}

// 通用的属性修改器
#[derive(Debug, Clone, Copy)]
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
    match (self.attribute, &self.value) {
    (Attribute::Health, AttributeValue::Float(v)) => {
      attributes.health = *v;
    }
    (Attribute::Energy, AttributeValue::Dur(v)) => {
      attributes.energy = *v;
    }
    (Attribute::Energy, AttributeValue::Float(v)) => {
      attributes.energy = seconds(v.round() as i64);
    }
    (Attribute::Water, AttributeValue::Dur(v)) => {
      attributes.water = *v;
    }
    (Attribute::Water, AttributeValue::Float(v)) => {
      attributes.water = seconds(v.round() as i64);
    }
    _ => (),
    }
  }
}
impl From<(Attribute, AttributeValue)> for AttributeModifier {
  fn from(value: (Attribute, AttributeValue)) -> Self {
    Self { attribute: value.0, value: value.1 }
  }
}

impl Default for Attributes {
  fn default() -> Self { Self::new() }
}

impl Attributes {
  pub const fn new() -> Self { Self { health: 1.0, energy: hours(72), water: hours(72) } }

  pub fn get(&self, attribute: Attribute) -> AttributeValue {
    match attribute {
    Attribute::Health => AttributeValue::Float(self.health),
    Attribute::Energy => AttributeValue::Dur(self.energy),
    Attribute::Water => AttributeValue::Dur(self.water),
    }
  }

  pub fn apply_modifier(&mut self, modifier: &AttributeModifier) { modifier.apply(self); }
}

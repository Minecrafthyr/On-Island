use std::borrow::Cow;

use crate::utils::NameAndDesc;

pub struct Damage {
  pub id: &'static str,
  pub amount: f64,
}

impl NameAndDesc for Damage {
  const PREFIX: &str = "damage";

  fn get_id(&self) -> &str { self.id }
}
impl Damage {
  pub fn new(id: &'static str, amount: f64) -> Self { Self { id, amount } }

  pub fn death_message(&self) -> Cow<'_, str> { t!(format!("{}.death", self.get_id())) }
}

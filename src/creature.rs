use crate::item::{Item, ItemDef, LIVING_FISH};

pub struct CreatureDef {
  pub id: &'static str,
  pub body: &'static ItemDef,
}
impl PartialEq for CreatureDef {
  fn eq(&self, other: &Self) -> bool { self.id == other.id }
}
#[derive(Clone)]
pub struct Creature {
  pub def: &'static CreatureDef,
  pub body: Item,
  pub worn: Vec<Item>,
}
impl From<&'static CreatureDef> for Creature {
  fn from(value: &'static CreatureDef) -> Self {
    Self { def: value, body: value.body.item(), worn: vec![] }
  }
}
pub const FISH: &CreatureDef = &CreatureDef { id: "fish", body: LIVING_FISH };

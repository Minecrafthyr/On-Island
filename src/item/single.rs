use time::Duration;

use super::*;
use crate::{builder_method, item::container::ContainerDef, player::Player, utils::NameAndDesc};
#[derive(Clone, Copy)]
pub struct UseData {
  pub dur: Duration,
  pub activity: f64,
  pub on_use: fn(&mut Player),
}
#[derive(Clone)]
pub struct ItemDef {
  pub id: &'static str,
  pub volume: Volume,
  pub weight: Mass,
  pub container_size: Volume,
  pub container: Option<ContainerDef>,
  pub use_data: Option<UseData>,
}

impl NameAndDesc for ItemDef {
  const PREFIX: &str = "item";

  fn get_id(&self) -> &str { self.id }
}

impl ItemDef {
  builder_method!(id, &'static str);

  builder_method!(volume, Volume);

  builder_method!(weight, Mass);

  builder_method!(container_size, Volume);

  builder_method!(use_data, Option<UseData>);

  builder_method!(container, Option<ContainerDef>);

  pub const fn new(id: &'static str, volume: Volume, weight: Mass) -> Self {
    Self { id, volume, weight, container_size: Default::default(), container: None, use_data: None }
  }

  pub const fn with_id(id: &'static str) -> Self {
    Self::new(id, Default::default(), Default::default())
  }
}
impl PartialEq for ItemDef {
  fn eq(&self, other: &Self) -> bool { self.id == other.id }
}
impl std::hash::Hash for ItemDef {
  fn hash<H: std::hash::Hasher>(&self, state: &mut H) { self.id.hash(state); }
}
#[derive(Clone, PartialEq)]
pub struct Item {
  pub def: &'static ItemDef,
  pub container: Option<Box<Container>>,
}

impl Item {
  pub const fn new(def: &'static ItemDef) -> Self { Self { def, container: None } }

  pub fn has_free_pockets(&self) -> bool {
    self
      .container
      .iter()
      .flat_map(|c| {
        c.pockets.iter().filter(|p| p.capacity > p.volume_used && p.max_weight > p.weight)
      })
      .count()
      > 0
  }

  pub fn get_weight(&self) -> Mass {
    self.weight
      + self
        .container
        .iter()
        .flat_map(|c| c.pockets.iter().map(|p| p.weight))
        .fold(Mass::ZERO, |m, p| m + p)
  }

  pub fn get_volume(&self) -> Volume {
    self.volume
      + self
        .container
        .iter()
        .flat_map(|c| c.pockets.iter().filter(|p| !p.rigid).map(|p| p.volume_used))
        .fold(Volume::ZERO, |v, p| v + p)
  }

  pub fn insert_items_from(&mut self, stack: &mut ItemStack) {
    let Some(container) = &mut self.container else { return };
    container.insert_items_from(stack);
  }

  pub fn insert_stacks_from(&mut self, stacks: &mut ItemStacks) {
    let Some(container) = &mut self.container else { return };
    container.insert_stacks_from(stacks);
  }
}

impl Deref for Item {
  type Target = ItemDef;

  fn deref(&self) -> &Self::Target { self.def }
}
impl From<&'static ItemDef> for Item {
  fn from(value: &'static ItemDef) -> Self {
    Self { def: value, container: value.container.as_ref().map(|d| Box::new(d.into())) }
  }
}

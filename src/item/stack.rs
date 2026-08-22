use std::fmt::Display;

use super::*;
use crate::utils::NameAndDesc;
#[derive(Clone, Copy, PartialEq)]
pub struct ItemDefStack {
  pub item: &'static ItemDef,
  pub count: u64,
}

impl ItemDefStack {
  pub const fn new(item: &'static ItemDef, count: u64) -> Self { Self { item, count } }
}
const impl From<(&'static ItemDef, u64)> for ItemDefStack {
  fn from(value: (&'static ItemDef, u64)) -> Self { Self { item: value.0, count: value.1 } }
}
impl Display for ItemDefStack {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{} ×{}", Item::new(self.item).name(), self.count)
  }
}
#[derive(Clone, PartialEq)]
pub struct ItemStack {
  pub item: Item,
  pub count: u64,
}

impl ItemStack {
  pub const fn new(item: Item, count: u64) -> Self { Self { item, count } }

  pub fn from_def(item: &'static ItemDef, count: u64) -> Self { Self { item: item.into(), count } }

  pub fn volume(&self) -> Volume { self.item.get_volume() * self.count }

  pub fn weight(&self) -> Mass { self.item.get_weight() * self.count }

  pub fn insert_stacks_separate(&mut self, stacks: &mut ItemStacks) -> Option<ItemStacks> {
    if !self.item.has_free_pockets() {
      return None;
    }
    let mut m = ItemStacks::new();
    for _ in 0..self.count {
      let mut item = self.item.clone();
      item.insert_stacks_from(stacks);
      m.insert_item(item);
    }
    Some(m)
  }

  pub fn insert_stacks_recursive(&mut self, mut stacks: ItemStacks) -> Option<ItemStacks> {
    if let Some(m) = self.insert_stacks_separate(&mut stacks) {
      stacks.insert_stacks(m);
    }
    Some(stacks)
  }
}
const impl From<(Item, u64)> for ItemStack {
  fn from(value: (Item, u64)) -> Self { Self::new(value.0, value.1) }
}
impl From<(&'static ItemDef, u64)> for ItemStack {
  fn from(value: (&'static ItemDef, u64)) -> Self { Self::new(value.0.into(), value.1) }
}
const impl From<ItemDefStack> for ItemStack {
  fn from(value: ItemDefStack) -> Self { Self { item: Item::new(value.item), count: value.count } }
}

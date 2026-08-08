use std::{
  fmt::Display,
  hash::Hash,
  mem::transmute,
  ops::{Deref, DerefMut, RangeBounds},
};

use time::Duration;

use crate::{player::Player, units::*, utils::NameAndDesc};
pub struct PocketDef {
  pub id: &'static str,
  pub capacity: Volume,
  pub max_weight: Mass,
  pub rigid: bool,
}
pub struct Pocket {
  pub def: &'static PocketDef,
  pub item_stacks: ItemStacks,
}
impl Deref for Pocket {
  type Target = PocketDef;

  fn deref(&self) -> &Self::Target { self.def }
}
pub struct ContainerDef {
  pub pocket_defs: &'static [PocketDef],
}

pub struct Container {
  pub def: &'static ContainerDef,
  pub pockets: Vec<Pocket>,
}
impl Deref for Container {
  type Target = ContainerDef;

  fn deref(&self) -> &Self::Target { self.def }
}
#[derive(Clone, Copy)]
pub struct UseData {
  pub dur: Duration,
  pub activity: f64,
  pub on_use: fn(&mut Player),
}
pub mod data;
pub use data::*;
impl NameAndDesc for ItemDef {
  const PREFIX: &str = "item";

  fn get_id(&self) -> &str { self.id }
}

#[derive(Clone, Copy, PartialEq, Hash)]
pub struct Item {
  pub def: &'static ItemDef,
}

impl Item {
  pub const fn new(def: &'static ItemDef) -> Self { Self { def } }
}

impl Deref for Item {
  type Target = ItemDef;

  fn deref(&self) -> &Self::Target { self.def }
}
const impl From<&'static ItemDef> for Item {
  fn from(value: &'static ItemDef) -> Self { Self { def: value } }
}

#[derive(Clone, Copy, PartialEq, Hash)]
pub struct ItemStack {
  pub item: Item,
  pub count: u64,
}

impl ItemStack {
  pub const fn new(item: Item, count: u64) -> Self { Self { item, count } }

  pub const fn from_def(item: &'static ItemDef, count: u64) -> Self {
    Self { item: item.into(), count }
  }

  pub fn volume(&self) -> Volume { self.item.volume * self.count }

  pub fn weight(&self) -> Mass { self.item.weight * self.count }
}
const impl From<(Item, u64)> for ItemStack {
  fn from(value: (Item, u64)) -> Self { Self::new(value.0, value.1) }
}
const impl From<(&'static ItemDef, u64)> for ItemStack {
  fn from(value: (&'static ItemDef, u64)) -> Self { Self::new(value.0.into(), value.1) }
}
const impl From<ItemDefStack> for ItemStack {
  fn from(value: ItemDefStack) -> Self { Self { item: Item::new(value.item), count: value.count } }
}

pub struct ItemMatcher<CountRange: RangeBounds<u64>> {
  pub defs: Vec<&'static ItemDef>,
  pub count: CountRange,
}
impl<CountRange: RangeBounds<u64>> ItemMatcher<CountRange> {
  pub fn matches(&self, items: ItemStack) -> bool {
    self.count.contains(&items.count) && self.defs.contains(&items.item.def)
  }
}

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
#[derive(Clone, Copy)]
#[derive_const(Default)]
pub struct ItemDefStacks(pub &'static [ItemDefStack]);
const impl Deref for ItemDefStacks {
  type Target = &'static [ItemDefStack];

  fn deref(&self) -> &Self::Target { &self.0 }
}
const impl DerefMut for ItemDefStacks {
  fn deref_mut(&mut self) -> &mut Self::Target { &mut self.0 }
}

const impl From<&'static [ItemDefStack]> for ItemDefStacks {
  fn from(value: &'static [ItemDefStack]) -> Self { Self(value) }
}
const impl<const S: usize> From<&'static [(&'static ItemDef, u64); S]> for ItemDefStacks {
  fn from(value: &'static [(&'static ItemDef, u64); S]) -> Self {
    Self(unsafe { transmute((value, S as u64)) })
  }
}
const impl<const S: usize> From<[(&'static ItemDef, u64); S]> for ItemDefStacks {
  fn from(value: [(&'static ItemDef, u64); S]) -> Self { Self(unsafe { transmute((&value, S)) }) }
}

#[derive(Clone)]
pub struct ItemStacks(pub Vec<ItemStack>);
const impl Deref for ItemStacks {
  type Target = Vec<ItemStack>;

  fn deref(&self) -> &Self::Target { &self.0 }
}
const impl DerefMut for ItemStacks {
  fn deref_mut(&mut self) -> &mut Self::Target { &mut self.0 }
}

const impl Default for ItemStacks {
  fn default() -> Self { Self::new() }
}

impl ItemStacks {
  pub const fn new() -> Self { Self(Vec::new()) }

  pub fn insert_item(&mut self, item: Item) {
    self
      .iter_mut()
      .find(|ei| ei.item == item)
      .map(|stack| stack.count += 1)
      .unwrap_or_else(|| self.push(ItemStack { item, count: 1 }));
  }

  pub fn insert_items(&mut self, item_stack: ItemStack) {
    self
      .iter_mut()
      .find(|ei| ei.item == item_stack.item)
      .map(|stack| stack.count += item_stack.count)
      .unwrap_or_else(|| self.push(item_stack));
  }

  pub fn count_of(&self, item: Item) -> u64 {
    self.iter().find(|ei| ei.item == item).map_or(0, |stack| stack.count)
  }

  pub fn remove_item(&mut self, item: Item) { self.remove_items(ItemStack { item, count: 1 }); }

  pub fn remove_items(&mut self, is: ItemStack) {
    let ItemStack { item, count } = is;
    let Some(i) = self.iter_mut().position(|ei| ei.item == item) else { return };
    self.0[i].count = self.0[i].count.saturating_sub(count);
    if self.0[i].count == 0 {
      self.0.swap_remove(i);
    }
  }
}

impl<I, C: IntoIterator<Item = I>> From<C> for ItemStacks
where
  ItemStack: From<I>,
{
  fn from(value: C) -> Self { Self::from_iter(value) }
}
impl<I> FromIterator<I> for ItemStacks
where
  ItemStack: From<I>,
{
  fn from_iter<T: IntoIterator<Item = I>>(iter: T) -> Self {
    Self(iter.into_iter().map(ItemStack::from).collect())
  }
}

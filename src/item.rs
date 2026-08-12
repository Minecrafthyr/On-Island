use std::ops::{Deref, DerefMut};

use crate::{item::container::Container, units::*};
pub mod container;
pub mod data;
pub use data::*;
pub mod item {
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
      Self {
        id,
        volume,
        weight,
        container_size: Default::default(),
        container: None,
        use_data: None,
      }
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
}
pub use item::*;

pub mod item_stack {
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

    pub fn from_def(item: &'static ItemDef, count: u64) -> Self {
      Self { item: item.into(), count }
    }

    pub fn volume(&self) -> Volume { self.item.volume * self.count }

    pub fn weight(&self) -> Mass { self.item.weight * self.count }
  }
  const impl From<(Item, u64)> for ItemStack {
    fn from(value: (Item, u64)) -> Self { Self::new(value.0, value.1) }
  }
  impl From<(&'static ItemDef, u64)> for ItemStack {
    fn from(value: (&'static ItemDef, u64)) -> Self { Self::new(value.0.into(), value.1) }
  }
  const impl From<ItemDefStack> for ItemStack {
    fn from(value: ItemDefStack) -> Self {
      Self { item: Item::new(value.item), count: value.count }
    }
  }
}
pub use item_stack::*;
pub mod item_stacks {
  use std::mem::transmute;

  use super::*;
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
  #[derive(Clone, PartialEq)]
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
    pub fn iter(&self) -> impl Iterator<Item = &ItemStack> { self.0.iter() }
  }

  impl ItemStacks {
    pub const fn new() -> Self { Self(Vec::new()) }

    pub fn insert_item(&mut self, item: Item) {
      if let Some(stack) = self.iter_mut().find(|ei| ei.item == item) {
        stack.count += 1;
      } else {
        self.push(ItemStack { item, count: 1 });
      }
    }

    pub fn insert_items(&mut self, item_stack: ItemStack) {
      if let Some(stack) = self.iter_mut().find(|ei| ei.item == item_stack.item) {
        stack.count += item_stack.count;
      } else {
        self.push(item_stack);
      }
    }

    pub fn count_of(&self, item: Item) -> u64 {
      self.iter().find(|ei| ei.item == item).map_or(0, |stack| stack.count)
    }

    pub fn remove_items(&mut self, item: &Item, count: u64) {
      self.remove_items_matching(|ei| ei == item, count);
    }

    pub fn remove_items_matching<F: Fn(&Item) -> bool>(&mut self, f: F, mut count: u64) {
      while let Some(i) = self.iter_mut().position(|ei| f(&ei.item)) {
        count -= self.0[i].count;
        self.0[i].count = self.0[i].count.saturating_sub(count);
        if self.0[i].count == 0 {
          self.0.swap_remove(i);
        }
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
}
pub use item_stacks::*;

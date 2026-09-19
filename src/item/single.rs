use std::ops::Mul;

use time::Duration;

use super::*;
use crate::{
  builder_method,
  item::container::{Pocket, PocketDef},
  location::RestorationData,
  player::Player,
  utils::NameAndDesc,
};
#[derive(Clone, Copy)]
pub struct UseData {
  pub dur: Duration,
  pub activity: f64,
  pub on_use: fn(&mut Player),
}

#[derive(Clone, Copy, PartialEq)]
pub enum Phase {
  Solid,
  Liquid,
  Gas,
}
/// TODO: expands to struct, with different requirements
#[derive(Clone, Copy)]
pub enum GatherStacks {
  None,
  Single(fn(&mut RestorationData) -> ItemStacks),
  Multiple(fn(&mut RestorationData, u64) -> ItemStacks),
}
#[derive(Clone)]
pub struct ItemDef {
  pub id: &'static str,
  pub volume: Volume,
  pub weight: Mass,
  pub phase: Phase,
  pub longest_side: Length,
  pub pockets: &'static [PocketDef],
  pub gather: GatherStacks,
  pub use_data: Option<UseData>,
}

const impl Mul<u64> for &'static ItemDef {
  type Output = ItemDefStack;

  fn mul(self, rhs: u64) -> Self::Output { ItemDefStack::new(self, rhs) }
}

impl NameAndDesc for ItemDef {
  const PREFIX: &str = "item";

  fn get_id(&self) -> &str { self.id }
}

impl ItemDef {
  builder_method!(id, &'static str);

  builder_method!(volume, Volume);

  builder_method!(weight, Mass);

  builder_method!(phase, Phase);

  builder_method!(gather, GatherStacks);

  builder_method!(use_data, Option<UseData>);

  builder_method!(pockets, &'static [PocketDef]);

  pub const fn new(id: &'static str, volume: Volume, weight: Mass) -> Self {
    Self {
      id,
      volume,
      weight,
      phase: Phase::Solid,
      longest_side: Length::ZERO,
      pockets: &[],
      gather: GatherStacks::None,
      use_data: None,
    }
  }

  pub fn item(&'static self) -> Item { Item::from(self) }

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
  pub pockets: Vec<Pocket>,
}

impl Mul<u64> for Item {
  type Output = ItemStack;

  fn mul(self, rhs: u64) -> Self::Output { ItemStack::new(self, rhs) }
}

impl Item {
  pub const fn new(def: &'static ItemDef) -> Self { Self { def, pockets: Vec::new() } }

  pub fn has_free_pockets(&self) -> bool {
    self.pockets.iter().filter(|p| p.capacity > p.volume_used && p.max_weight > p.weight).count()
      > 0
  }

  pub fn get_weight(&self) -> Mass {
    self.weight + self.pockets.iter().map(|p| p.weight).fold(Mass::ZERO, |m, p| m + p)
  }

  pub fn get_volume(&self) -> Volume {
    self.volume
      + self
        .pockets
        .iter()
        .filter(|p| !p.rigid)
        .map(|p| p.volume_used)
        .fold(Volume::ZERO, |v, p| v + p)
  }

  pub fn insert_stack_from(&mut self, stack: &mut ItemStack) {
    if !self.has_free_pockets() {
      return;
    }
    while let Some(pocket) = self.pockets.iter_mut().find(|p| p.holdable_count(stack) > 0) {
      pocket.insert_stack_from(stack);
    }
  }

  pub fn insert_stacks_from(&mut self, stacks: &mut ItemStacks) {
    if !self.has_free_pockets() {
      return;
    }
    for stack in stacks.iter_mut() {
      self.insert_stack_from(stack);
    }
    *stacks = std::mem::take(stacks).0.into_iter().filter(|stack| stack.count != 0).collect();
  }
}

impl Deref for Item {
  type Target = ItemDef;

  fn deref(&self) -> &Self::Target { self.def }
}
impl From<&'static ItemDef> for Item {
  fn from(value: &'static ItemDef) -> Self {
    Self { def: value, pockets: value.pockets.iter().map(|d| d.into()).collect() }
  }
}

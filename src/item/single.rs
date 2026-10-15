use std::{borrow::Cow, ops::Mul};

use enumflags2::BitFlags;
use inew::New;

use super::*;
use crate::{
  item::container::{Pocket, PocketDef},
  location::{EnvCondition, EnvData, ResData},
  player::Player,
  utils::{NameAndDesc, TakeItems},
};
#[derive(Clone, Copy)]
pub struct UseData {
  pub usage: &'static str,
  pub dur: Duration,
  pub batch: (usize, f64),
  pub activity: f64,
  pub on_use: fn(&mut Player),
}
impl NameAndDesc for UseData {
  const PREFIX: &str = "use";

  fn get_id(&self) -> Cow<'_, str> { self.usage.into() }
}

#[derive(Clone, Copy, PartialEq)]
pub enum Phase {
  Solid,
  Liquid,
  Gas,
}
type SingleResFn = fn(&mut ResData) -> ItemStacks;
type MultipleResFn = fn(&mut ResData, u64) -> ItemStacks;
/// TODO: expands to struct, with different requirements
#[derive(Clone, Copy)]
pub enum GatherStacks {
  None,
  Single(SingleResFn),
  Multiple(MultipleResFn),
}
impl From<SingleResFn> for GatherStacks {
  fn from(value: SingleResFn) -> Self { Self::Single(value) }
}
impl From<MultipleResFn> for GatherStacks {
  fn from(value: MultipleResFn) -> Self { Self::Multiple(value) }
}

#[allow(unpredictable_function_pointer_comparisons)]
#[derive(PartialEq, New)]
#[new(const = true)]
pub struct ConversionDef {
  cond: EnvCondition,
  dur: Duration,
  into: fn() -> ItemStacks,
}

#[derive(Clone, PartialEq)]
pub struct Conversion {
  def: &'static ConversionDef,
  progress: Duration,
}
impl From<&'static ConversionDef> for Conversion {
  fn from(value: &'static ConversionDef) -> Self { Self { def: value, progress: Duration::ZERO } }
}
// #[derive(PartialEq, New)]
// #[new(const = true)]
// pub struct CreatureDef {}

// pub struct Creature {
//   def: &'static CreatureDef,
// }

#[derive(Clone)]
pub struct ItemDef {
  pub id: &'static str,
  pub volume: Volume,
  pub weight: Mass,
  pub phase: Phase = Phase::Solid,
  pub longest_side: Length = Length::ZERO,
  pub pockets: &'static [PocketDef] = &[] as &'static [PocketDef],
  pub gather: GatherStacks= GatherStacks::None,
  pub uses: &'static [UseData] = &[] as &'static [UseData],
  pub conversions: &'static [ConversionDef] = &[] as &'static [ConversionDef],
}

const impl Mul<u64> for &'static ItemDef {
  type Output = ItemDefStack;

  fn mul(self, rhs: u64) -> Self::Output { ItemDefStack::new(self, rhs) }
}

impl NameAndDesc for ItemDef {
  const PREFIX: &str = "item";

  fn get_id(&self) -> Cow<'_, str> { self.id.into() }
}

impl ItemDef {
  pub fn item(&'static self) -> Item { Item::from(self) }
}
impl PartialEq for ItemDef {
  fn eq(&self, other: &Self) -> bool { self.id == other.id }
}
impl std::hash::Hash for ItemDef {
  fn hash<H: std::hash::Hasher>(&self, state: &mut H) { self.id.hash(state); }
}
#[derive(Clone, PartialEq, New)]
#[new(const = true)]
pub struct Item {
  pub def: &'static ItemDef,
  #[new(default = vec![])]
  pub pockets: Vec<Pocket>,
  #[new(default = vec![])]
  pub conv: Vec<Conversion>,
}

const impl Mul<u64> for Item {
  type Output = ItemStack;

  fn mul(self, rhs: u64) -> Self::Output { ItemStack::new(self, rhs) }
}

impl Item {
  // TODO: improve tick items
  pub fn tick(&mut self, env: BitFlags<EnvData>) -> Option<ItemStacks> {
    for Conversion { def, progress } in &mut self.conv {
      let ConversionDef { cond, dur, into } = def;
      if cond.matches(env) {
        *progress += 1.ms();
      }
      if *progress >= *dur {
        return Some(into());
      }
    }
    None
  }

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

  pub fn insert_stack_from(&mut self, stack: &mut ItemStack) -> u64 {
    let mut count = 0u64;
    if !self.has_free_pockets() {
      return 0;
    }
    while let Some(pocket) = self.pockets.iter_mut().find(|p| p.holdable_count(stack) > 0) {
      count += pocket.insert_stack_from(stack);
    }
    count
  }

  pub fn insert_stacks_from(&mut self, stacks: &mut ItemStacks) -> u64 {
    let mut count = 0u64;
    if !self.has_free_pockets() {
      return 0;
    }
    for stack in stacks.iter_mut() {
      count += self.insert_stack_from(stack);
    }
    *stacks = std::mem::take(stacks).0.into_iter().filter(|stack| stack.count != 0).collect();
    count
  }
}
impl<F: Fn(&Item) -> bool> TakeItems<F> for Item {
  fn take_items(&mut self, f: F, mut count: u64) -> (ItemStacks, u64) {
    let mut removed_total = ItemStacks::new();
    for p in &mut self.pockets {
      let (removed, mismatch) = p.take_items(&f, count);
      removed_total.insert_stacks(removed);
      count = mismatch;
      if count == 0 {
        break;
      }
    }
    (removed_total, count)
  }
}

impl Deref for Item {
  type Target = ItemDef;

  fn deref(&self) -> &Self::Target { self.def }
}
impl From<&'static ItemDef> for Item {
  fn from(value: &'static ItemDef) -> Self {
    Self {
      def: value,
      pockets: value.pockets.iter().map(|d| d.into()).collect(),
      conv: value.conversions.iter().map(|c| c.into()).collect(),
    }
  }
}

use std::{borrow::Cow, ops::Mul};

use enumflags2::BitFlags;

use super::*;
use crate::{
  builder_method,
  item::container::{Pocket, PocketDef},
  location::{ResData, WaterYield},
  player::Player,
  utils::NameAndDesc,
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
#[derive(PartialEq, Eq, Clone, Copy)]
pub enum Required {
  Yes,
  No,
  None,
}
impl From<bool> for Required {
  fn from(value: bool) -> Self { if value { Self::Yes } else { Self::No } }
}
impl From<Option<bool>> for Required {
  fn from(value: Option<bool>) -> Self {
    if let Some(cond) = value { if cond { Self::Yes } else { Self::No } } else { Self::None }
  }
}
#[derive(PartialEq)]
pub enum Condition {
  AlwaysTrue,
  AirExposure(bool),
  Gather { dirt: Option<bool>, sand: Option<bool>, water: Option<BitFlags<WaterYield>> },
  AllOf(&'static [Self]),
  AnyOf(&'static [Self]),
}
#[derive(Clone, Copy)]
pub struct EnvContext {
  air: bool,
  dirt: bool,
  sand: bool,
  water: WaterYield,
}
impl Condition {
  pub fn matches(&self, env: &EnvContext) -> bool {
    use Condition::*;
    match self {
    AlwaysTrue => true,
    Gather { dirt, sand, water } => todo!(),
    AirExposure(a) => *a == env.air,
    AllOf(conditions) => conditions.iter().all(|c| c.matches(env)),
    AnyOf(conditions) => conditions.iter().any(|c| c.matches(env)),
    }
  }
}
#[allow(unpredictable_function_pointer_comparisons)]
#[derive(PartialEq)]
pub struct ConversionDef {
  cond: Condition,
  dur: Duration,
  into: fn() -> ItemStacks,
}
impl ConversionDef {
  pub const fn new(cond: Condition, dur: Duration, into: fn() -> ItemStacks) -> Self {
    Self { cond, dur, into }
  }
}

#[derive(Clone, PartialEq)]
pub struct Conversion {
  def: &'static ConversionDef,
  progress: Duration,
}
impl From<&'static ConversionDef> for Conversion {
  fn from(value: &'static ConversionDef) -> Self { Self { def: value, progress: Duration::ZERO } }
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
  pub uses: &'static [UseData],
  pub conversions: &'static [ConversionDef],
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
  pub const fn new(id: &'static str, volume: Volume, weight: Mass) -> Self {
    Self {
      id,
      volume,
      weight,
      phase: Phase::Solid,
      longest_side: Length::ZERO,
      pockets: &[],
      gather: GatherStacks::None,
      uses: &[],
      conversions: &[],
    }
  }
}

impl ItemDef {
  builder_method! {id, &'static str}

  builder_method! {volume, Volume}

  builder_method! {weight, Mass}

  builder_method! {phase, Phase}

  builder_method! {gather, GatherStacks}

  builder_method! {uses, &'static [UseData]}

  builder_method! {pockets, &'static [PocketDef]}

  builder_method! {conversions, &'static [ConversionDef]}

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
  pub conv: Vec<Conversion>,
}

const impl Mul<u64> for Item {
  type Output = ItemStack;

  fn mul(self, rhs: u64) -> Self::Output { ItemStack::new(self, rhs) }
}

impl Item {
  pub const fn new(def: &'static ItemDef) -> Self { Self { def, pockets: vec![], conv: vec![] } }

  pub fn tick(&mut self, env: &EnvContext) -> ItemStacks {
    for Conversion { def, progress } in &mut self.conv {
      let ConversionDef { cond, dur, into } = def;
      if cond.matches(env) {
        *progress += 1.ms();
      }
      if *progress >= *dur {
        return into();
      }
    }
    ItemStacks::new()
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

  pub fn take_items_matching(
    &mut self, f: impl Fn(&Item) -> bool, mut count: u64,
  ) -> (ItemStacks, u64) {
    let mut removed_total = ItemStacks::new();
    for p in &mut self.pockets {
      let (removed, mismatch) = p.take_items_matching(&f, count);
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

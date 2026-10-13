use std::ops::{Index, IndexMut, RangeInclusive};

use Location::*;
use enumflags2::{BitFlags, bitflags};
use inew::New;
use rand::{Rng, RngExt};
use strum::EnumCount;
use strum_macros::{EnumCount, EnumIter, EnumString, IntoStaticStr};

use crate::{
  item::{Item, ItemDef, ItemStack, ItemStacks},
  preclude::*,
  utils::NameAndDesc,
};

#[bitflags]
#[repr(u16)]
#[derive(Copy, Clone, PartialEq, Eq)]
pub enum EnvData {
  NaturalAir,
  CaveAir,
  NoAir,
  Dry,
  Moist,
  Wet,
  Puddle,
  Overflow,
  // ---
  Shallow,
  FlowingShallow,
  Deep,
  FlowingDeep,
  Bottomless,
  FlowingBottomless,
  Sand,
  Dirt,
  // ---
}
#[derive(PartialEq)]
pub enum EnvCondition {
  AlwaysTrue,
  Data(BitFlags<EnvData>),
  AllOf(&'static [Self]),
  AnyOf(&'static [Self]),
}
impl EnvCondition {
  pub fn matches(&self, env: BitFlags<EnvData>) -> bool {
    use EnvCondition::*;
    match self {
    AlwaysTrue => true,
    Data(e) => env.contains(*e),
    AllOf(conditions) => conditions.iter().all(|c| c.matches(env)),
    AnyOf(conditions) => conditions.iter().any(|c| c.matches(env)),
    }
  }
}

pub struct Locations(pub [LocationData; Location::COUNT]);
#[derive(Clone, EnumCount, EnumIter, Copy, PartialEq, Eq, Hash, EnumString, IntoStaticStr)]
#[strum(serialize_all = "snake_case")]
pub enum Location {
  StrandedShip,
  Beach,
  ShallowSea,
  Forest,
}
impl Locations {
  pub fn new() -> Self {
    use EnvData::*;

    use crate::item::*;
    Self([
      LocationData::new(
        NaturalAir | Dry,
        [WATER_BOTTLE.clone() * 10, BISCUIT.item() * 10, CANVAS_BACKPACK.item() * 1],
        ItemStacks::new(),
        vec![],
        vec![(Beach, 30.s(), &[]), (ShallowSea, 1.mnt(), &[PLASTIC_BOTTLE])],
      ),
      LocationData::new(
        NaturalAir | Dry | Wet | FlowingShallow,
        [ROCK.item() * 50],
        [RAW_FISH.item() * 10],
        vec![],
        vec![
          (StrandedShip, 40.s(), &[]),
          (Forest, 3.mnt(), &[]),
          (ShallowSea, 1.mnt(), &[PLASTIC_BOTTLE]),
        ],
      ),
      LocationData::new(
        NaturalAir | Wet | FlowingShallow,
        [ROCK.item() * 56],
        [RAW_FISH.item() * 10, BIG_ROCK.item() * 80],
        vec![ResData::new(RAW_FISH, 1..=2, 0.2, 20, 20.mnt(), 120.s()..=1200.s(), 1.2)],
        vec![(Beach, 1.mnt(), &[PLASTIC_BOTTLE, RAW_FISH])],
      ),
      LocationData::new(
        NaturalAir | Dry | Moist,
        ItemStacks::new(),
        [TREE.item() * 500, TREE_VINE.item() * 400],
        vec![
          ResData::new(TREE, 1..=1, 1.0, 600, 60.day(), 120.s()..=1200.s(), 1.2),
          ResData::new(TREE_VINE, 1..=1, 0.8, 1000, 10.day(), 10.s()..=30.s(), 1.1),
        ],
        vec![(Beach, 3.mnt(), &[])],
      ),
    ])
  }
}
#[derive(Clone, New)]
pub struct ResData {
  #[new(into)]
  pub item: Item,
  pub attempts: RangeInclusive<u64>,
  pub chance: f64,
  pub limit: u64,
  pub interval: Duration,
  #[new(default)]
  pub last_time: Duration,
  pub gather_time: RangeInclusive<Duration>,
  pub activity: f64,
}
pub struct LocationData {
  pub env: BitFlags<EnvData>,
  pub pickup_stacks: ItemStacks,
  pub gather_stacks: ItemStacks,
  pub res: Vec<ResData>,
  pub can_go: Vec<(Location, Duration, &'static [&'static ItemDef])>,
}
impl LocationData {
  pub fn new(
    env: BitFlags<EnvData>, pickup_stacks: impl IntoIterator<Item = ItemStack>,
    gather_stacks: impl IntoIterator<Item = ItemStack>, res: Vec<ResData>,
    can_go: Vec<(Location, Duration, &'static [&'static ItemDef])>,
  ) -> Self {
    Self {
      env,
      pickup_stacks: pickup_stacks.into_iter().collect(),
      gather_stacks: gather_stacks.into_iter().collect(),
      res,
      can_go,
    }
  }

  pub fn tick(&mut self, time: Duration, rng: &mut impl Rng) {
    self.gather_stacks = self
      .gather_stacks
      .iter()
      .cloned()
      .flat_map(|mut s| s.item.tick(self.env).unwrap_or([s].into_iter().collect()))
      .collect();
    for r in &mut self.res {
      let next_time = r.last_time + r.interval;
      if next_time <= time {
        r.last_time = next_time;
        let mut count = 0;
        for _ in 0..rng.random_range(r.attempts.clone()) {
          if rng.random_bool(r.chance) {
            count += 1;
          }
        }
        self.gather_stacks.insert_items(&r.item, count);
      }
    }
  }
}
impl NameAndDesc for Location {
  const PREFIX: &str = "location";

  fn get_id(&self) -> Cow<'_, str> { Cow::Borrowed(self.into()) }
}

impl Index<Location> for Locations {
  type Output = LocationData;

  fn index(&self, index: Location) -> &Self::Output { &self.0[index as usize] }
}
impl IndexMut<Location> for Locations {
  fn index_mut(&mut self, index: Location) -> &mut Self::Output { &mut self.0[index as usize] }
}
impl Default for Locations {
  fn default() -> Self { Self::new() }
}

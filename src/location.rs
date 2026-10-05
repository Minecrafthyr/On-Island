use std::ops::{Index, IndexMut, RangeInclusive};

use enumflags2::bitflags;
use rand::{Rng, RngExt};
use strum::EnumCount;
use strum_macros::{EnumCount, EnumIter, EnumString, IntoStaticStr};

use crate::{
  item::{Item, ItemDef, ItemStacks},
  preclude::*,
  utils::NameAndDesc,
};
#[bitflags]
#[repr(u16)]
#[derive(Copy, Clone, PartialEq, Eq)]
pub enum WaterYield {
  Dry,
  Wet,
  Puddle,
  Overflow,
  Shallow,
  FlowingShallow,
  Deep,
  FlowingDeep,
  Bottomless,
  FlowingBottomless,
}
#[derive(Clone, EnumCount, EnumIter, Copy, PartialEq, Eq, Hash, EnumString, IntoStaticStr)]
#[strum(serialize_all = "snake_case")]
pub enum Location {
  StrandedShip,
  Beach,
  ShallowSea,
  Forest,
}
pub struct Locations(pub [LocationData; Location::COUNT]);
impl Locations {
  pub fn new() -> Self {
    use crate::item::*;
    Self([
      LocationData::new(
        [WATER_BOTTLE.clone() * 10, BISCUIT.item() * 10, CANVAS_BACKPACK.item() * 1],
        ItemStacks::new(),
        vec![],
        vec![(Beach, 30.s(), &[]), (ShallowSea, 1.mnt(), &[PLASTIC_BOTTLE])],
      ),
      LocationData::new([ROCK * 50], [RAW_FISH * 10], vec![], vec![
        (StrandedShip, 40.s(), &[]),
        (Forest, 3.mnt(), &[]),
        (ShallowSea, 1.mnt(), &[PLASTIC_BOTTLE]),
      ]),
      LocationData::new(
        [ROCK * 56],
        [RAW_FISH * 10, BIG_ROCK * 80],
        vec![ResData::new(RAW_FISH, 1..=2, 0.2, 20, 20.mnt(), 120.s()..=1200.s(), 1.2)],
        vec![(Beach, 1.mnt(), &[PLASTIC_BOTTLE, RAW_FISH])],
      ),
      LocationData::new(
        ItemStacks::new(),
        [TREE * 500, TREE_VINE * 400],
        vec![
          ResData::new(TREE, 1..=1, 1.0, 600, 60.day(), 120.s()..=1200.s(), 1.2),
          ResData::new(TREE_VINE, 1..=1, 0.8, 1000, 10.day(), 10.s()..=30.s(), 1.1),
        ],
        vec![(Beach, 3.mnt(), &[])],
      ),
    ])
  }
}
#[derive(Clone)]
pub struct ResData {
  pub item: Item,
  pub attempts: RangeInclusive<u64>,
  pub chance: f64,
  pub limit: u64,
  pub interval: Duration,
  pub last_time: Duration,
  pub gather_time: RangeInclusive<Duration>,
  pub activity: f64,
}

impl ResData {
  pub fn new(
    item: impl Into<Item>, attempts: RangeInclusive<u64>, chance: f64, limit: u64,
    interval: Duration, gather_time: RangeInclusive<Duration>, activity: f64,
  ) -> Self {
    Self {
      item: item.into(),
      attempts,
      chance,
      limit,
      interval,
      last_time: Duration::ZERO,
      gather_time,
      activity,
    }
  }
}
// pub struct Transfer {
//   items: &'static [&'static ItemDef],
// }
pub struct LocationData {
  pub pickup_stacks: ItemStacks,
  pub gather_stacks: ItemStacks,
  pub res: Vec<ResData>,
  pub can_go: Vec<(Location, Duration, &'static [&'static ItemDef])>,
}
impl LocationData {
  pub fn new(
    pickup_stacks: impl Into<ItemStacks>, gather_stacks: impl Into<ItemStacks>, res: Vec<ResData>,
    can_go: Vec<(Location, Duration, &'static [&'static ItemDef])>,
  ) -> Self {
    Self { pickup_stacks: pickup_stacks.into(), gather_stacks: gather_stacks.into(), res, can_go }
  }

  pub fn tick(&mut self, time: Duration, rng: &mut impl Rng) {
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
        self.gather_stacks.insert_stack_from(&r.item, count);
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
use Location::*;

use std::ops::{Index, IndexMut, RangeInclusive};

use rand::{Rng, RngExt};
use strum::EnumCount;
use strum_macros::{EnumCount, EnumIter, EnumString, IntoStaticStr};

use crate::{
  builder_method,
  item::{Item, ItemStacks},
  preclude::*,
  utils::NameAndDesc,
};

#[derive(Clone)]
pub struct ResData {
  pub item: Item,
  pub attempts: RangeInclusive<u64>,
  pub chance: f64,
  pub limit: u64,
  pub interval: Duration,
  pub last_time: Duration,
  pub count: u64,
  pub gather_time: RangeInclusive<Duration>,
  pub activity: f64,
  pub turn_into: Option<(&'static ResData, u64)>,
}

impl ResData {
  builder_method! {turn_into,Option<(&'static ResData, u64)>}

  pub fn new(
    item: impl Into<Item>, attempts: RangeInclusive<u64>, chance: f64, limit: u64,
    interval: Duration, count: u64, gather_time: RangeInclusive<Duration>, activity: f64,
  ) -> Self {
    Self {
      item: item.into(),
      attempts,
      chance,
      limit,
      interval,
      last_time: Duration::ZERO,
      count,
      gather_time,
      activity,
      turn_into: None,
    }
  }
}
pub struct LocationData {
  pub pickup_stacks: ItemStacks,
  pub restore_and_gather: Vec<ResData>,
  pub can_go: Vec<(Location, Duration)>,
}
impl LocationData {
  pub fn new(
    pickup_stacks: impl Into<ItemStacks>, restore_and_gather: Vec<ResData>,
    can_go: Vec<(Location, Duration)>,
  ) -> Self {
    Self { pickup_stacks: pickup_stacks.into(), restore_and_gather, can_go }
  }

  pub fn tick(&mut self, time: Duration, rng: &mut impl Rng) {
    for r in &mut self.restore_and_gather {
      let next_time = r.last_time + r.interval;
      if next_time <= time {
        r.last_time = next_time;
        let mut count = 0;
        for _ in 0..rng.random_range(r.attempts.clone()) {
          if rng.random_bool(r.chance) {
            count += 1;
          }
        }
        r.count += count;
      }
    }
  }
}
#[derive(Clone, EnumCount, EnumIter, Copy, PartialEq, Eq, Hash, EnumString, IntoStaticStr)]
#[strum(serialize_all = "snake_case")]
pub enum Location {
  StrandedShip,
  Beach,
  Forest,
}
impl NameAndDesc for Location {
  const PREFIX: &str = "location";

  fn get_id(&self) -> Cow<'_, str> { Cow::Borrowed(self.into()) }
}

pub struct Locations(pub [LocationData; Location::COUNT]);
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
impl Locations {
  pub fn new() -> Self {
    use crate::item::*;
    Self([
      LocationData::new(
        [WATER_BOTTLE.clone() * 10, BISCUIT.item() * 10, CANVAS_BACKPACK.item() * 1],
        vec![],
        vec![(Beach, 30.s())],
      ),
      LocationData::new(
        [(BIG_ROCK, 50)],
        Vec::from([ResData::new(RAW_FISH, 1..=2, 0.3, 10, 20.mnt(), 10, 120.s()..=1200.s(), 1.2)]),
        vec![(StrandedShip, 40.s()), (Forest, 3.mnt())],
      ),
      LocationData::new(
        ItemStacks(vec![]),
        Vec::from([
          ResData::new(TREE, 1..=1, 1.0, 600, 60.day(), 500, 120.s()..=1200.s(), 1.2),
          ResData::new(TREE_VINE, 1..=1, 0.8, 1000, 10.day(), 400, 10.s()..=30.s(), 1.1),
        ]),
        vec![(Beach, 3.mnt())],
      ),
    ])
  }
}

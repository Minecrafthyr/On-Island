use std::ops::{Index, IndexMut, RangeInclusive};

use rand::{Rng, RngExt};
use strum::EnumCount;
use strum_macros::{EnumCount, EnumIter, EnumString, IntoStaticStr};
use time::Duration;

use crate::{
  item::{Item, ItemStack, ItemStacks},
  utils::NameAndDesc,
};
#[derive(Debug)]
pub struct RestorationData {
  pub item: Item,
  pub times: RangeInclusive<u64>,
  pub chance: f64,
  pub limit: u64,
  pub interval: Duration,
  pub last_time: Duration,
}
impl RestorationData {
  pub fn new(
    item: Item, times: RangeInclusive<u64>, chance: f64, limit: u64, interval: Duration,
  ) -> Self {
    Self { item, times, chance, limit, interval, last_time: Duration::ZERO }
  }
}
#[derive(Debug)]
pub struct LocationData {
  pub pickup_stacks: ItemStacks,
  pub gather_stacks: ItemStacks,
  pub restore: Vec<RestorationData>,
  pub can_go: Vec<(Location, Duration)>,
}
impl LocationData {
  pub fn tick(&mut self, time: Duration, rng: &mut impl Rng) {
    for r in &mut self.restore {
      let next_time = r.last_time + r.interval;
      if next_time <= time {
        r.last_time = next_time;
        let times = rng.random_range(r.times.clone());
        let mut count = 0;
        for _ in 0..times {
          if rng.random_bool(r.chance) {
            count += 1;
          }
        }
        self.gather_stacks.insert_items(ItemStack { item: r.item, count });
      }
    }
  }
}
#[derive(
  Debug, Clone, EnumCount, EnumIter, Copy, PartialEq, Eq, Hash, EnumString, IntoStaticStr,
)]
#[strum(serialize_all = "snake_case")]
pub enum Location {
  StrandedShip,
  Beach,
  Forest,
}
impl NameAndDesc for Location {
  const PREFIX: &str = "location";

  fn get_id(&self) -> &str { self.into() }
}

#[derive(Debug)]
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
impl Locations {
  pub fn new() -> Self {
    use crate::{item::*, units::*};
    Self([
      LocationData {
        pickup_stacks: [(BISCUIT, 10), (WATER, 10), (CANVAS_BACKPACK, 1)].into(),
        gather_stacks: ItemStacks::new(),
        restore: vec![],
        can_go: vec![(Location::Beach, seconds(30))],
      },
      LocationData {
        pickup_stacks: [(ROCK, 50)].into(),
        gather_stacks: [(RAW_FISH, 10)].into(),
        restore: vec![RestorationData::new(RAW_FISH.into(), 1..=2, 0.3, 10, minutes(20))],
        can_go: vec![(Location::StrandedShip, seconds(40)), (Location::Forest, minutes(3))],
      },
      LocationData {
        pickup_stacks: ItemStacks::new(),
        gather_stacks: [(WOOD, 400), (TREE_VINE, 500)].into(),
        restore: vec![
          RestorationData::new(WOOD.into(), 1..=1, 1.0, 500, days(10)),
          RestorationData::new(TREE_VINE.into(), 1..=1, 0.8, 500, days(1)),
        ],
        can_go: vec![(Location::Beach, minutes(3))],
      },
    ])
  }
}

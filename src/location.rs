use std::ops::{Index, IndexMut, RangeInclusive};

use rand::{Rng, RngExt};
use strum::EnumCount;
use strum_macros::{EnumCount, EnumIter, EnumString, IntoStaticStr};
use time::Duration;

use crate::{
  item::{Item, ItemStacks},
  utils::NameAndDesc,
};

#[derive(Clone)]
pub struct RestorationData {
  pub item: Item,
  pub attempts: RangeInclusive<u64>,
  pub chance: f64,
  pub limit: u64,
  pub interval: Duration,
  pub last_time: Duration,
  pub count: u64,
  pub gather_time: RangeInclusive<Duration>,
  pub activity: f64,
}
pub struct LocationData {
  pub pickup_stacks: ItemStacks,
  pub restore_and_gather: Vec<RestorationData>,
  pub can_go: Vec<(Location, Duration)>,
}
impl LocationData {
  pub fn new(
    pickup_stacks: impl Into<ItemStacks>, restore_and_gather: Vec<RestorationData>,
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

  fn get_id(&self) -> &str { self.into() }
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
impl Locations {
  pub fn new() -> Self {
    use crate::{item::*, units::*};
    Self([
      LocationData::new(
        [WATER_BOTTLE.clone() * 10, BISCUIT.item() * 10, CANVAS_BACKPACK.item() * 1],
        vec![],
        vec![(Location::Beach, 30.sec())],
      ),
      LocationData::new(
        [(BIG_ROCK, 50)],
        vec![RestorationData {
          item: RAW_FISH.into(),
          attempts: 1..=2,
          chance: 0.3,
          limit: 10,
          interval: 20.minute(),
          last_time: Duration::ZERO,
          count: 10,
          gather_time: 120.sec()..=1200.sec(),
          activity: 1.2,
        }],
        vec![(Location::StrandedShip, 40.sec()), (Location::Forest, 3.minute())],
      ),
      LocationData::new(
        ItemStacks(vec![]),
        vec![
          RestorationData {
            item: TREE.into(),
            attempts: 1..=1,
            chance: 1.0,
            limit: 600,
            interval: 60.day(),
            last_time: Duration::ZERO,
            count: 500,
            gather_time: 120.sec()..=1200.sec(),
            activity: 1.2,
          },
          RestorationData {
            item: TREE_VINE.into(),
            attempts: 1..=1,
            chance: 0.8,
            limit: 1000,
            interval: 10.day(),
            last_time: Duration::ZERO,
            count: 400,
            gather_time: 10.sec()..=30.sec(),
            activity: 1.1,
          },
        ],
        vec![(Location::Beach, 3.minute())],
      ),
    ])
  }
}

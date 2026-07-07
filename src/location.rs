use std::ops::{Index, IndexMut, RangeInclusive};

use rand::{Rng, RngExt};
use strum::EnumCount;
use strum_macros::{EnumCount, EnumIter, EnumString, IntoStaticStr};
use time::Duration;

use super::*;
#[derive(Debug)]
pub struct RestorationData {
    pub item: Item,
    pub count: RangeInclusive<u64>,
    pub interval: Duration,
    pub last_time: Duration,
    pub chance: f64,
    pub limit: u64,
}
#[derive(Debug)]
pub struct LocationData {
    pub pickup_stacks: ItemStacks,
    pub gather_stacks: ItemStacks,
    pub restore: Vec<RestorationData>,
}
impl LocationData {
    pub fn tick(&mut self, time: Duration, rng: &mut impl Rng) {
        for r in &mut self.restore {
            let next_time = r.last_time + r.interval;
            if next_time <= time && rng.random_bool(r.chance) {
                r.last_time = next_time;
                if let Some(ei) = self.gather_stacks.iter_mut().find(|is| is.item == r.item) {
                    ei.count += rng.random_range(r.count.clone());
                } else {
                    self.gather_stacks
                        .push(ItemStack { item: r.item, count: rng.random_range(r.count.clone()) });
                }
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
        use item::Item::*;
        Self([
            LocationData {
                pickup_stacks: [(Biscuit, 10u64), (Water, 10u64)].into(),
                gather_stacks: ItemStacks::new(),
                restore: vec![],
            },
            LocationData {
                pickup_stacks: [(Rock, 50)].into(),
                gather_stacks: [(RawFish, 10)].into(),
                restore: vec![RestorationData {
                    item: RawFish,
                    count: 1..=2,
                    interval: Duration::minutes(20),
                    last_time: Duration::ZERO,
                    chance: 0.3,
                    limit: 10,
                }],
            },
            LocationData {
                pickup_stacks: ItemStacks::new(),
                gather_stacks: [(Wood, 400)].into(),
                restore: vec![RestorationData {
                    item: Wood,
                    count: 1..=1,
                    interval: Duration::days(10),
                    last_time: Duration::ZERO,
                    chance: 1.0,
                    limit: 500,
                }],
            },
        ])
    }
}

#[derive(Debug)]
pub struct Connection {
    pub a: Location,
    pub b: Location,
    pub time_a_to_b: Duration,
    pub time_b_to_a: Duration,
}

impl Connection {
    pub fn other_side(&self, l: Location) -> Option<(Location, Duration)> {
        if l == self.a {
            Some((self.b, self.time_a_to_b))
        } else if l == self.b {
            Some((self.a, self.time_b_to_a))
        } else {
            None
        }
    }
}

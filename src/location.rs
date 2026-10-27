use std::ops::{Index, IndexMut, RangeInclusive};

use Location::*;
use enumflags2::{BitFlags, bitflags};
use rand::{Rng, RngExt};
use strum::EnumCount;
use strum_macros::{EnumCount, EnumIter, EnumString, IntoStaticStr};

use crate::{
  creature::{Creature, CreatureDef, FISH},
  item::{Item, ItemDef, ItemStacks},
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
  // ---
  Dry,
  Moist,
  Wet,
  Puddle,
  Overflow,
  Shallow,
  FlowingShallow,
  Deep,
  FlowingDeep,
  Bottomless,
  FlowingBottomless,
  // ---
  Sand,
  Dirt,
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

#[derive(Clone, EnumCount, EnumIter, Copy, PartialEq, Eq, Hash, EnumString, IntoStaticStr)]
#[strum(serialize_all = "snake_case")]
pub enum Location {
  StrandedShip,
  Beach,
  ShallowSea,
  Forest,
}
pub struct Locations(pub [LocationData; Location::COUNT]);
impl Default for Locations {
  fn default() -> Self {
    use EnvData::*;

    use crate::item::*;
    let stranded_ship = LocationData {
      env: NaturalAir | Dry,
      pickup_stacks: ItemStacks(vec![water_bottle() * 10, BISCUIT * 10, CANVAS_BACKPACK * 1]),
      can_go: vec![(Beach, 30.s(), &[]), (ShallowSea, 1.mnt(), &[PLASTIC_BOTTLE])],
      ..
    };
    let beach = LocationData {
      env: NaturalAir | Dry | Wet | FlowingShallow,
      pickup_stacks: ItemStacks(vec![ROCK * 50]),
      gather_stacks: ItemStacks(vec![LIVING_FISH * 10]),
      can_go: vec![
        (StrandedShip, 40.s(), &[]),
        (Forest, 3.mnt(), &[]),
        (ShallowSea, 1.mnt(), &[PLASTIC_BOTTLE]),
      ],
      ..
    };
    let shallow_sea = LocationData {
      env: NaturalAir | Wet | FlowingShallow,
      pickup_stacks: ItemStacks(vec![ROCK * 56]),
      gather_stacks: ItemStacks(vec![LIVING_FISH * 10, BIG_ROCK * 80]),
      can_go: vec![(Beach, 1.mnt(), &[PLASTIC_BOTTLE, LIVING_FISH])],
      spawn: vec![(FISH, Gen { attempts: 1..=2, chance: 0.2, limit: 20, interval: 20.mnt(), .. })],
      ..
    };
    let forest = LocationData {
      env: NaturalAir | Dry | Moist,
      pickup_stacks: ItemStacks(vec![ROCK * 12, BRANCH * 20]),
      gather_stacks: ItemStacks(vec![TREE * 500, TREE_VINE * 400]),
      resource: vec![
        (TREE.into(), Gen { attempts: 1..=1, chance: 1.0, limit: 600, interval: 60.day(), .. }),
        (TREE_VINE.into(), Gen {
          attempts: 1..=1,
          chance: 0.8,
          limit: 1000,
          interval: 10.day(),
          ..
        }),
      ],
      can_go: vec![(Beach, 3.mnt(), &[])],
      ..
    };
    Self([stranded_ship, beach, shallow_sea, forest])
  }
}
#[derive(Clone)]
pub struct Gen {
  pub attempts: RangeInclusive<u64>,
  pub chance: f64,
  pub limit: u64,
  pub interval: Duration,
  pub last_time: Duration = Duration::ZERO,
}
impl Gen {
  pub fn tick(&mut self, time: Duration, rng: &mut impl Rng, existing: u64) -> u64 {
    let next_time = self.last_time + self.interval;
    if next_time <= time {
      self.last_time = next_time;
      let attempts = rng.random_range(self.attempts.clone()) as f64;
      let add = rng.random_range((attempts * self.chance)..attempts) as u64;
      (existing + add).min(self.limit)
    } else {
      0
    }
  }
}
pub struct LocationData {
  pub env: BitFlags<EnvData>,
  pub pickup_stacks: ItemStacks = ItemStacks::new(),
  pub gather_stacks: ItemStacks = ItemStacks::new(),
  pub resource: Vec<(Item, Gen)> = vec![],
  pub creatures: Vec<Creature> = vec![],
  pub spawn: Vec<(&'static CreatureDef, Gen)> = vec![],
  pub can_go: Vec<(Location, Duration, &'static [&'static ItemDef])> = vec![],
}
impl LocationData {
  pub fn tick(&mut self, time: Duration, rng: &mut impl Rng) {
    let converted_gather = self.gather_stacks.tick(self.env);
    self.gather_stacks += converted_gather;
    let converted_pickups = self.pickup_stacks.tick(self.env);
    self.pickup_stacks += converted_pickups;
    for (i, p) in &mut self.resource {
      if let Some(is) = self.gather_stacks.iter_mut().find(|is| is.item == *i) {
        is.count = p.tick(time, rng, is.count);
      } else {
        self.gather_stacks.push(i.clone() * p.tick(time, rng, 0));
      }
    }
    for (cref, g) in &mut self.spawn {
      let cdef = *cref;
      let creature_count = self.creatures.iter().filter(|c| c.def == cdef).count() as u64;
      for _ in 0..g.tick(time, rng, creature_count) {
        self.creatures.push(cdef.into());
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

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn mob_generation_respects_limit() {
    let mut location = LocationData {
      env: EnvData::NaturalAir.into(),
      spawn: vec![(FISH, Gen { attempts: 2..=2, chance: 0.5, limit: 1, interval: 1.ms(), .. })],
      ..
    };
    let mut rng = rand::rng();

    location.tick(1.ms(), &mut rng);
    location.tick(2.ms(), &mut rng);

    assert_eq!(location.creatures.iter().filter(|creature| creature.def == FISH).count(), 1);
  }
}

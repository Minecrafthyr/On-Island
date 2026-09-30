use std::{error::Error, fmt::Display, sync::Arc};

use itertools::Itertools;
use strum_macros::{EnumCount, EnumIter, EnumString, IntoStaticStr};

pub use crate::{
  damage::Damage,
  game::end_game,
  item::{CANVAS_BACKPACK, COTTON_PANTIES, COTTON_UNDERWEAR, Item, ItemStack, ItemStacks},
  location::Location,
  player::action::Action,
  preclude::*,
  units::*,
};
pub mod action;

pub struct BodyPart {
  pub id: &'static str,
}
impl PartialEq for BodyPart {
  fn eq(&self, other: &Self) -> bool { self.id == other.id }
}
impl NameAndDesc for BodyPart {
  const PREFIX: &str = "body_part";

  fn get_id(&self) -> Cow<'_, str> { self.id.into() }
}
type IsExclusive = bool;
pub enum Effect {
  ActionSpeedMul(f64),
  ActivityMul(f64),
  EnergyConsumeMul(f64),
  WaterConsumeMul(f64),
  HealthRegenMul(f64),
}

pub struct Player {
  pub health: f64,
  pub energy: Duration,
  pub water: Duration,
  pub location: Location,
  pub actions: Vec<Action>,
  pub used_body_parts: Vec<(&'static BodyPart, IsExclusive)>,
  pub effects: Vec<Effect>,
  pub worn: Vec<Item>,
}

impl Default for Player {
  fn default() -> Self { Self::new() }
}

impl Player {
  pub fn get_activity(&self) -> f64 {
    self.effects.iter().fold(1.0, |base, effect| {
      if let Effect::ActivityMul(mul) = effect { base * mul } else { base }
    })
  }
}
impl Player {
  pub fn new() -> Self {
    Player {
      health: 1.0,
      energy: 72.h(),
      water: 72.h(),
      location: Location::StrandedShip,
      actions: vec![],
      used_body_parts: vec![],
      effects: vec![],
      worn: vec![CANVAS_BACKPACK.into(), COTTON_PANTIES.into(), COTTON_UNDERWEAR.into()],
    }
  }

  pub fn get_efficiency(&self) -> f64 {
    let mut base = 1.0;
    if self.energy < 24.h() {
      base *= self.energy.as_s_f64() / 24.h().as_s_f64();
    }
    if self.water < 24.h() {
      base *= self.water.as_s_f64() / 24.h().as_s_f64();
    }
    if self.health < 0.5 {
      base *= self.health / 0.5;
    }
    base
  }

  pub fn damage(&mut self, dmg: Damage) {
    self.health -= dmg.amount;
    if self.health < 0.0 {
      end_game(dmg.death_message());
    }
  }

  pub fn tick(&mut self) {
    let dur = 1.ms() * self.get_activity();
    self.energy -= dur;
    self.water -= dur;
    self.tick_actions();

    if self.energy <= 24.h() {
      self.damage(Damage::new("starve", 0.001));
    }
    if self.water <= 24.h() {
      self.damage(Damage::new("dehydrate", 0.001));
    }

    if self.energy > 48.h() && self.water > 48.h() && self.health < 1.0 {
      self.energy -= dur;
      self.health += 0.001;
    }
  }

  pub fn max_carry_weight(&self) -> Mass { kg(60) * self.get_efficiency() }

  pub fn max_pickup_weight(&self) -> Mass { kg(20) * self.get_efficiency() }

  pub fn max_carry_volume(&self) -> Volume {
    L(10) + self.worn.iter().fold(Volume::ZERO, |v, i| v + i.volume)
  }

  pub fn worn_volume_total(&self) -> Volume {
    self.worn.iter().map(|i| i.get_volume()).fold(Volume::ZERO, |v, iv| v + iv)
  }

  pub fn worn_weight_total(&self) -> Mass {
    self.worn.iter().map(|i| i.get_weight()).fold(Mass::ZERO, |w, iw| w + iw)
  }

  pub fn worn_volume_contained(&self) -> Volume {
    self.worn.iter().flat_map(|i| &i.pockets).fold(Volume::ZERO, |v, p| v + p.volume_used)
  }

  pub fn worn_weight_contained(&self) -> Mass {
    self.worn.iter().flat_map(|i| &i.pockets).fold(Mass::ZERO, |v, p| v + p.weight)
  }

  pub fn insert_stack_from(&mut self, stack: &mut ItemStack) {
    for i in &mut self.worn {
      i.insert_stack_from(stack);
      if stack.count == 0 {
        return;
      }
    }
  }

  pub fn insert_stack(&mut self, mut stack: ItemStack) -> Option<ItemStack> {
    self.insert_stack_from(&mut stack);
    if stack.count == 0 { None } else { Some(stack) }
  }

  pub fn insert_stacks_from(&mut self, stacks: &mut ItemStacks) {
    for i in &mut self.worn {
      i.insert_stacks_from(stacks);
      if stacks.is_empty() {
        return;
      }
    }
  }

  pub fn insert_stacks(&mut self, mut stacks: ItemStacks) -> Option<ItemStacks> {
    self.insert_stacks_from(&mut stacks);
    if stacks.is_empty() { None } else { Some(stacks) }
  }

  pub fn take_items_matching<F: Fn(&Item) -> bool>(
    &mut self, f: F, mut count: u64,
  ) -> (ItemStacks, u64) {
    let mut removed = ItemStacks::new();
    for p in self.worn.iter_mut().flat_map(|i| i.pockets.iter_mut()) {
      let (r, m) = p.take_items_matching(&f, count);
      removed.insert_stacks(r);
      count = m;
    }
    (removed, count)
  }

  pub fn pickup_time(&self, item: &Item) -> Option<(Duration, f64)> {
    if item.weight > self.max_pickup_weight() {
      return None;
    }
    let weight = item.weight.as_kg_f64().max(0.0);

    let base_time = weight * 0.1 + 0.6;
    let base_activity = weight * 0.1 + 1.2;
    let weight_factor = if weight > 0.0 { 1.0 + (weight / 10.0).ln().max(0.0) * 0.1 } else { 1.0 };

    let activity_factor = 1.0 + (weight / 5.0).sqrt() * 0.3;
    let activity = (base_activity * activity_factor) / self.get_efficiency();

    let time_seconds = base_time * weight_factor * self.get_efficiency();

    Some((Duration::from_s_f64(time_seconds), activity))
  }

  pub fn get_inventory(&self) -> Vec<&ItemStack> {
    self.worn.iter().flat_map(|i| i.pockets.iter()).flat_map(|p| p.stacks.iter()).collect()
  }

  pub fn count_of_matching<F: Fn(&Item) -> bool>(&self, f: F) -> u64 {
    self.get_inventory().into_iter().filter(|ei| f(&ei.item)).map(|stack| stack.count).sum()
  }

  pub fn count_of(&self, item: &Item) -> u64 { self.count_of_matching(|ei| ei == item) }
}

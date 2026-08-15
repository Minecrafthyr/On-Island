use itertools::Itertools;
use time::Duration;

use crate::{
  item::{
    CANVAS_BACKPACK, COTTON_PANTIES, COTTON_UNDERWEAR, Item, ItemStack, ItemStacks,
    container::Pocket,
  },
  location::Location,
  units::*,
};

pub struct DamageDef {
  pub death_message: &'static str,
  pub log_message: &'static str,
}

pub struct Damage {
  pub def: DamageDef,
  pub amount: f64,
}

pub struct Player {
  pub health: f64,
  pub energy: Duration,
  pub water: Duration,
  pub location: Location,
  pub activity: f64,
  pub inventory_volume_used: Volume,
  pub inventory_weight: Mass,
  pub worn: Vec<Item>,
}

impl Default for Player {
  fn default() -> Self { Self::new() }
}

impl Player {
  pub fn new() -> Self {
    Player {
      health: 1.0,
      energy: hours(72),
      water: hours(72),
      location: Location::StrandedShip,
      activity: 1.0,
      inventory_volume_used: Volume::ZERO,
      inventory_weight: Mass::ZERO,
      worn: vec![CANVAS_BACKPACK.into(), COTTON_PANTIES.into(), COTTON_UNDERWEAR.into()],
    }
  }

  pub fn efficiency(&self) -> f64 {
    let mut base = 1.0;
    if self.energy < Duration::hours(24) {
      base *= self.energy.as_seconds_f64() / Duration::hours(24).as_seconds_f64();
    }
    if self.water < Duration::hours(24) {
      base *= self.water.as_seconds_f64() / Duration::hours(24).as_seconds_f64();
    }
    if self.health < 0.5 {
      base *= self.health / 0.5;
    }
    base
  }

  pub fn tick(&mut self, activity: f64) {
    let dur = Duration::MILLISECOND * activity;
    self.energy -= dur;
    self.water -= dur;

    if self.energy <= Duration::hours(24) {
      self.health -= 0.001;
    }
    if self.water <= Duration::hours(24) {
      self.health -= 0.001;
    }

    if self.energy > Duration::hours(48) && self.water > Duration::hours(48) && self.health < 1.0 {
      self.energy -= dur;
      self.health += 0.001;
    }
  }

  pub fn max_carry_weight(&self) -> Mass { kg(60) * self.efficiency() }

  pub fn max_pickup_weight(&self) -> Mass { kg(20) * self.efficiency() }

  pub fn max_carry_volume(&self) -> Volume {
    L(10) + self.worn.iter().fold(Volume::ZERO, |v, i| v + i.volume)
  }

  pub fn free_pockets(&mut self) -> Vec<&mut Pocket> {
    self
      .worn
      .iter_mut()
      .filter_map(|i| i.container.as_mut())
      .flat_map(|c| {
        c.pockets.iter_mut().filter(|p| p.capacity > p.volume_used && p.max_weight > p.weight)
      })
      .sorted_by(|a, b| Ord::cmp(&a.priority, &b.priority))
      .collect()
  }

  pub fn worn_containers_volume_used(&self) -> Volume {
    self
      .worn
      .iter()
      .filter_map(|i| i.container.as_ref())
      .fold(Volume::ZERO, |v, c| v + c.pockets.iter().fold(Volume::ZERO, |v, p| v + p.volume_used))
  }

  pub fn insert_items(&mut self, mut i: ItemStack) -> Option<ItemStack> {
    for p in self.free_pockets() {
      p.insert_items_from(&mut i);
      if i.count == 0 {
        return None;
      }
    }
    Some(i)
  }

  pub fn insert_items_from(&mut self, i: &mut ItemStack) {
    for p in self.free_pockets() {
      p.insert_items_from(i);
      if i.count == 0 {
        return;
      }
    }
  }

  pub fn insert_stacks(&mut self, mut i: ItemStacks) -> Option<ItemStacks> {
    for p in self.free_pockets() {
      p.insert_stacks_from(&mut i);
      if i.is_empty() {
        return None;
      }
    }
    Some(i)
  }

  pub fn insert_stacks_from(&mut self, i: &mut ItemStacks) {
    for p in self.free_pockets() {
      p.insert_stacks_from(i);
      if i.is_empty() {
        return;
      }
    }
  }

  pub fn remove_items_matching<F: Fn(&Item) -> bool>(
    &mut self, f: F, count: u64,
  ) -> (Vec<ItemStack>, u64) {
    let mut removed = Vec::new();
    let mut mismatch = 0u64;
    for p in
      self.worn.iter_mut().filter_map(|i| i.container.as_mut()).flat_map(|c| c.pockets.iter_mut())
    {
      let (mut r, m) = p.remove_items_matching(&f, count);
      removed.append(&mut r);
      mismatch += m;
    }
    for is in &removed {
      self.inventory_volume_used -= is.volume();
      self.inventory_weight -= is.weight();
    }
    (removed, mismatch)
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
    let activity = (base_activity * activity_factor) / self.efficiency();

    let time_seconds = base_time * weight_factor * self.efficiency();
    Some((Duration::seconds_f64(time_seconds), activity))
  }

  pub fn get_inventory(&self) -> Vec<&ItemStack> {
    self
      .worn
      .iter()
      .filter_map(|i| i.container.as_ref())
      .flat_map(|c| c.pockets.iter())
      .flat_map(|p| p.stacks.iter())
      .collect()
  }

  pub fn count_of_matching<F: Fn(&Item) -> bool>(&self, f: F) -> u64 {
    self.get_inventory().iter().filter(|ei| f(&ei.item)).map(|stack| stack.count).sum()
  }

  pub fn count_of(&self, item: &Item) -> u64 { self.count_of_matching(|ei| ei == item) }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn new_player_starts_with_full_stats() {
    let player = Player::new();
    assert!((player.health - 1.0).abs() < f64::EPSILON);
    assert_eq!(player.energy, hours(72));
    assert_eq!(player.water, hours(72));
  }

  #[test]
  fn tick_decreases_resources() {
    let mut player = Player::new();
    player.tick(1.0);
    assert!(player.energy < hours(72));
    assert!(player.water < hours(72));
  }
}

use time::Duration;

use crate::{
  item::{CANVAS_BACKPACK, COTTON_PANTIES, COTTON_UNDERWEAR, Item, ItemStack, ItemStacks},
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

  pub fn worn_volume_total(&self) -> Volume {
    self.worn.iter().map(|i| i.get_volume()).fold(Volume::ZERO, |v, iv| v + iv)
  }

  pub fn worn_weight_total(&self) -> Mass {
    self.worn.iter().map(|i| i.get_weight()).fold(Mass::ZERO, |w, iw| w + iw)
  }

  pub fn worn_volume_contained(&self) -> Volume {
    self.worn.iter().flat_map(|i| i.pockets.iter()).fold(Volume::ZERO, |v, p| v + p.volume_used)
  }

  pub fn worn_weight_contained(&self) -> Mass {
    self.worn.iter().flat_map(|i| i.pockets.iter()).fold(Mass::ZERO, |v, p| v + p.weight)
  }

  pub fn insert_items_from(&mut self, stack: &mut ItemStack) {
    for i in &mut self.worn {
      i.insert_items_from(stack);
      if stack.count == 0 {
        return;
      }
    }
  }

  pub fn insert_items(&mut self, mut stack: ItemStack) -> Option<ItemStack> {
    self.insert_items_from(&mut stack);
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

  pub fn remove_items_matching<F: Fn(&Item) -> bool>(
    &mut self, f: F, count: u64,
  ) -> (Vec<ItemStack>, u64) {
    let mut removed = Vec::new();
    let mut mismatch = 0u64;
    for p in self.worn.iter_mut().flat_map(|i| i.pockets.iter_mut()) {
      let (mut r, m) = p.remove_items_matching(&f, count);
      removed.append(&mut r);
      mismatch += m;
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
    self.worn.iter().flat_map(|i| i.pockets.iter()).flat_map(|p| p.stacks.iter()).collect()
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

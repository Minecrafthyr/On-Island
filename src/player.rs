use time::Duration;

use crate::{
  attribute::Attributes,
  item::{CANVAS_BACKPACK, Item, ItemStack, ItemStacks},
  location::Location,
  units::*,
};

pub struct Player {
  pub attrs: Attributes,
  pub location: Location,
  pub inventory: ItemStacks,
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
      attrs: Attributes::new(),
      location: Location::StrandedShip,
      inventory: ItemStacks::new(),
      activity: 1.0,
      inventory_volume_used: Volume::ZERO,
      inventory_weight: Mass::ZERO,
      worn: vec![CANVAS_BACKPACK.into()],
    }
  }

  pub fn efficiency(&self) -> f64 {
    let mut base = 1.0;
    if self.attrs.energy < Duration::hours(24) {
      base *= self.attrs.energy.as_seconds_f64() / Duration::hours(24).as_seconds_f64();
    }
    if self.attrs.water < Duration::hours(24) {
      base *= self.attrs.water.as_seconds_f64() / Duration::hours(24).as_seconds_f64();
    }
    if self.attrs.health < 0.5 {
      base *= self.attrs.health / 0.5;
    }
    base
  }

  pub fn tick(&mut self, activity: f64) {
    let dur = Duration::MILLISECOND * activity;
    self.attrs.energy -= dur;
    self.attrs.water -= dur;

    if self.attrs.energy <= Duration::hours(24) {
      self.attrs.health -= 0.001;
    }
    if self.attrs.water <= Duration::hours(24) {
      self.attrs.health -= 0.001;
    }

    if self.attrs.energy > Duration::hours(48)
      && self.attrs.water > Duration::hours(48)
      && self.attrs.health < 1.0
    {
      self.attrs.energy -= dur;
      self.attrs.health += 0.001;
    }
  }

  pub fn max_carry_weight(&self) -> Mass { kg(60) * self.efficiency() }

  pub fn max_pickup_weight(&self) -> Mass { kg(20) * self.efficiency() }

  pub fn max_carry_volume(&self) -> Volume {
    L(10) + self.worn.iter().fold(Volume::ZERO, |v, i| v + i.volume)
  }

  pub fn recalc_volume_and_size(&mut self) {
    self.inventory_volume_used = self.inventory.iter().fold(Volume::ZERO, |v, i| v + i.volume());
    self.inventory_weight = self.inventory.iter().fold(Mass::ZERO, |v, i| v + i.weight());
  }

  pub fn remove_item(&mut self, item: Item) {
    self.inventory.remove_item(item);
    self.recalc_volume_and_size();
  }

  pub fn remove_items(&mut self, is: ItemStack) {
    self.inventory.remove_items(is);
    self.recalc_volume_and_size();
  }

  pub fn insert_item(&mut self, i: Item) -> Option<Item> {
    let new_volume = self.inventory_volume_used + i.volume;
    if !(new_volume < self.max_carry_volume()) {
      return Some(i);
    };
    let new_weight = self.inventory_weight + i.weight;
    if !(new_weight < self.max_carry_weight()) {
      return Some(i);
    };
    self.inventory.insert_item(i);
    self.inventory_volume_used = new_volume;
    self.inventory_weight = new_weight;
    None
  }

  pub fn insert_items(&mut self, i: ItemStack) -> Option<ItemStack> {
    let volume_left = self.max_carry_volume() - self.inventory_volume_used;
    let can_hold: u64 = (volume_left / i.item.volume.into()).into();
    let mut count = can_hold;
    if can_hold == 0u64 {
      return Some(i);
    } else if can_hold < i.count {
      count = can_hold;
    };
    let weight_left: Mass = self.max_carry_weight() - self.inventory_weight;
    let can_hold = (weight_left / i.item.weight.into()).into();
    if can_hold == 0u64 {
      return Some(i);
    } else if can_hold < i.count {
      count = count.min(can_hold);
    };
    self.inventory_volume_used += i.item.volume * count;
    self.inventory_weight += i.item.weight * count;
    self.inventory.insert_items(ItemStack { item: i.item, count });
    if i.count != count { Some(ItemStack { item: i.item, count: i.count - count }) } else { None }
  }

  pub fn pickup_time(&self, item: Item) -> Option<(Duration, f64)> {
    if item.weight > self.max_pickup_weight() {
      return None;
    }
    let weight = item.weight.as_kg_f64().max(0.0);

    let (base_time, base_activity) = match weight {
    w if w < 0.5 => (0.5, 1.2),
    w if w < 5.0 => (1.0, 1.8),
    w if w < 20.0 => (1.5, 2.5),
    w if w < 50.0 => (2.5, 4.0),
    _ => (4.0, 6.0),
    };

    let weight_factor = if weight > 0.0 { 1.0 + (weight / 10.0).ln().max(0.0) * 0.1 } else { 1.0 };

    let activity_factor = 1.0 + (weight / 5.0).sqrt() * 0.3;
    let activity = (base_activity * activity_factor) / self.efficiency();

    let time_seconds = base_time * weight_factor * self.efficiency();
    Some((Duration::seconds_f64(time_seconds), activity))
  }
}

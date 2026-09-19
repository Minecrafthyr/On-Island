use std::{error::Error, fmt::Display, sync::Arc};

use itertools::Itertools;
use strum_macros::{EnumCount, EnumIter, EnumString, IntoStaticStr};
use time::Duration;

use crate::{
  item::{CANVAS_BACKPACK, COTTON_PANTIES, COTTON_UNDERWEAR, Item, ItemStack, ItemStacks},
  location::Location,
  units::*,
  utils::NameAndDesc,
};

pub struct DamageDef {
  pub death_message: &'static str,
  pub log_message: &'static str,
}

pub struct Damage {
  pub def: DamageDef,
  pub amount: f64,
}

pub struct BodyPart {
  pub id: &'static str,
}
impl PartialEq for BodyPart {
  fn eq(&self, other: &Self) -> bool { self.id == other.id }
}
impl NameAndDesc for BodyPart {
  const PREFIX: &str = "body_part";

  fn get_id(&self) -> &str { self.id }
}
type IsExclusive = bool;
pub struct ActionContent {
  pub body_parts: Vec<(&'static BodyPart, IsExclusive)>,
  pub effects: Vec<Effect>,
}
#[derive(Clone)]
pub struct Action {
  pub id: &'static str,
  pub func: Arc<dyn Fn(&mut Player) -> ActionContent>,
  pub progress: Duration,
  pub total_dur: Duration,
}
impl Action {
  pub fn no_progress(
    id: &'static str, f: impl Fn(&mut Player) -> ActionContent + 'static, total_dur: Duration,
  ) -> Self {
    Self { id, func: Arc::new(f), progress: Duration::ZERO, total_dur }
  }
}
pub enum Effect {
  ActionSpeedMul(f64),
  ActivityMul(f64),
  EnergyConsumeMul(f64),
  WaterConsumeMul(f64),
  HealthRegenerateMul(f64),
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

  fn eval_actions(&mut self) {
    let funcs: Vec<_> = self.actions.iter().map(|a| a.func.clone()).collect();
    'func: for f in funcs {
      let ActionContent { body_parts, effects } = (f)(self);
      for (bp, is_exclusive) in body_parts {
        if let Some((_, e_is_exclusive)) =
          self.used_body_parts.iter_mut().find(|(ebp, _)| *ebp == bp)
        {
          if *e_is_exclusive {
            continue 'func;
          } else if is_exclusive {
            *e_is_exclusive = true;
          }
        } else {
          self.used_body_parts.push((bp, is_exclusive));
        }
      }
      self.effects.extend(effects);
    }
  }

  fn tick_actions(&mut self) {
    let mut action_idx = 0;
    while action_idx < self.actions.len() {
      let efficiency = 1.milliseconds() * self.get_efficiency();
      let action = &mut self.actions[action_idx];
      action.progress += efficiency;
      if action.progress > action.total_dur {
        self.actions.remove(action_idx);
        self.eval_actions();
      } else {
        action_idx += 1;
      }
    }
  }
}
#[derive(
  Debug, Clone, EnumCount, EnumIter, Copy, PartialEq, Eq, Hash, EnumString, IntoStaticStr,
)]
#[strum(serialize_all = "snake_case")]
pub enum PushActionError {
  BodyPartUsed,
}
impl NameAndDesc for PushActionError {
  const PREFIX: &str = "wait_action_result";

  fn get_id(&self) -> &str { self.into() }
}
impl Error for PushActionError {}
impl Display for PushActionError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { f.write_str(&self.name()) }
}
impl Player {
  pub fn try_push_action(&mut self, action: Action) -> Result<(), PushActionError> {
    let ActionContent { body_parts, effects: _ } = (action.func)(self);
    for (bp, _is_exclusive) in body_parts {
      if self.used_body_parts.iter_mut().contains(&(bp, true)) {
        return Err(PushActionError::BodyPartUsed);
      }
    }
    Ok(())
  }

  pub fn new() -> Self {
    Player {
      health: 1.0,
      energy: 72.hours(),
      water: 72.hours(),
      location: Location::StrandedShip,
      actions: vec![],
      used_body_parts: vec![],
      effects: vec![],
      worn: vec![CANVAS_BACKPACK.into(), COTTON_PANTIES.into(), COTTON_UNDERWEAR.into()],
    }
  }

  pub fn get_efficiency(&self) -> f64 {
    let mut base = 1.0;
    if self.energy < 24.hours() {
      base *= self.energy.as_seconds_f64() / 24.hours().as_seconds_f64();
    }
    if self.water < 24.hours() {
      base *= self.water.as_seconds_f64() / 24.hours().as_seconds_f64();
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
    self.tick_actions();

    if self.energy <= 24.hours() {
      self.health -= 0.001;
    }
    if self.water <= 24.hours() {
      self.health -= 0.001;
    }

    if self.energy > 48.hours() && self.water > 48.hours() && self.health < 1.0 {
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
    self.worn.iter().flat_map(|i| i.pockets.iter()).fold(Volume::ZERO, |v, p| v + p.volume_used)
  }

  pub fn worn_weight_contained(&self) -> Mass {
    self.worn.iter().flat_map(|i| i.pockets.iter()).fold(Mass::ZERO, |v, p| v + p.weight)
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
    let activity = (base_activity * activity_factor) / self.get_efficiency();

    let time_seconds = base_time * weight_factor * self.get_efficiency();
    Some((Duration::seconds_f64(time_seconds), activity))
  }

  pub fn get_inventory(&self) -> Vec<&ItemStack> {
    self.worn.iter().flat_map(|i| i.pockets.iter()).flat_map(|p| p.stacks.iter()).collect()
  }

  pub fn count_of_matching<F: Fn(&Item) -> bool>(&self, f: F) -> u64 {
    self.get_inventory().into_iter().filter(|ei| f(&ei.item)).map(|stack| stack.count).sum()
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
    assert_eq!(player.energy, 72.hours());
    assert_eq!(player.water, 72.hours());
  }

  #[test]
  fn tick_decreases_resources() {
    let mut player = Player::new();
    player.tick(1.0);
    assert!(player.energy < 72.hours());
    assert!(player.water < 72.hours());
  }
}

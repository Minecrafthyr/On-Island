use super::*;
use crate::utils::TakeItems;

pub struct CloseData {}
pub struct PocketDef {
  pub id: &'static str,
  pub capacity: Volume,
  pub max_weight: Mass,
  pub rigid: bool,
  pub can_store_liquid: bool= false,
  pub can_store_gas: bool= false,
  pub specify_items: Option<fn(&Item) -> bool>= None,
}

impl PartialEq for PocketDef {
  fn eq(&self, other: &Self) -> bool { self.id == other.id }
}
impl Eq for PocketDef {}
#[derive(Clone)]
pub struct Pocket {
  pub def: &'static PocketDef,
  pub stacks: ItemStacks = ItemStacks::new(),
  pub volume_used: Volume =  Volume::ZERO,
  pub weight: Mass = Mass::ZERO,
  pub priority: i32 = 0,
  pub closed: bool = true,
}

impl PartialEq for Pocket {
  fn eq(&self, other: &Self) -> bool { self.def == other.def }
}
impl Deref for Pocket {
  type Target = PocketDef;

  fn deref(&self) -> &Self::Target { self.def }
}
impl From<&'static PocketDef> for Pocket {
  fn from(value: &'static PocketDef) -> Self { Self { def: value, .. } }
}
impl Pocket {
  pub fn recalc_volume_and_size(&mut self) {
    self.volume_used = self.stacks.iter().fold(Volume::ZERO, |v, i| v + i.volume());
    self.weight = self.stacks.iter().fold(Mass::ZERO, |v, i| v + i.weight());
  }

  pub fn holdable_count_i(&self, item: &Item, count: u64) -> u64 {
    if self.closed
      || (!self.def.can_store_liquid && item.phase == Phase::Liquid)
      || self.def.specify_items.map(|f| f(item)) == Some(false)
    {
      return 0;
    }
    let volume_cap: u64 = ((self.capacity - self.volume_used) / item.volume.into()).into();
    let max_hold = volume_cap.min(count);
    if max_hold == 0 {
      return 0;
    }
    let weight_cap = ((self.max_weight - self.weight) / item.weight.into()).into();
    max_hold.min(weight_cap)
  }

  pub fn holdable_count(&self, stack: &ItemStack) -> u64 {
    self.holdable_count_i(&stack.item, stack.count)
  }

  pub fn insert_items(&mut self, item: &Item, count: u64) -> u64 {
    let count = self.holdable_count_i(item, count);
    if count == 0 {
      return 0;
    }
    self.volume_used += item.volume * count;
    self.weight += item.weight * count;
    self.stacks += item.clone() * count;
    count
  }

  pub fn insert_stack_from(&mut self, stack: &mut ItemStack) -> u64 {
    let count = self.holdable_count(stack);
    if count == 0 {
      return 0;
    }
    self.volume_used += stack.item.volume * count;
    self.weight += stack.item.weight * count;
    stack.count -= count;
    self.stacks += ItemStack { item: stack.item.clone(), count };
    count
  }

  pub fn insert_stack(&mut self, mut stack: ItemStack) -> Option<ItemStack> {
    self.insert_stack_from(&mut stack);
    if stack.count == 0 { None } else { Some(stack) }
  }

  pub fn insert_stacks_from(&mut self, stacks: &mut ItemStacks) {
    for stack in 0..stacks.len() {
      self.insert_stack_from(&mut stacks[stack]);
      if stacks[stack].count == 0 {
        stacks.remove(stack);
      }
    }
  }

  pub fn insert_stacks(&mut self, mut stacks: ItemStacks) -> Option<ItemStacks> {
    self.insert_stacks_from(&mut stacks);
    if stacks.is_empty() { None } else { Some(stacks) }
  }
}
impl TakeItems<&Item> for Pocket {
  fn take_items(&mut self, item: &Item, count: u64) -> (ItemStacks, u64) {
    self.take_items(|ei: &Item| ei == item, count)
  }
}
impl<F: Fn(&Item) -> bool> TakeItems<F> for Pocket {
  fn take_items(&mut self, f: F, count: u64) -> (ItemStacks, u64) {
    let (removed, mismatch) = self.stacks.take_items(f, count);
    for is in &removed.0 {
      self.volume_used -= is.volume();
      self.weight -= is.weight();
    }
    (removed, mismatch)
  }
}

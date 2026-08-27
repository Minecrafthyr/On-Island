use super::*;

pub struct CloseData {}

pub struct PocketDef {
  pub id: &'static str,
  pub capacity: Volume,
  pub max_weight: Mass,
  pub rigid: bool,
  pub can_store_liquid: bool,
  // pub can_store_gas: bool,
  pub specify_items: Option<fn(&Item) -> bool>,
}

impl PartialEq for PocketDef {
  fn eq(&self, other: &Self) -> bool { self.id == other.id }
}
impl Eq for PocketDef {}
#[derive(Clone)]
pub struct Pocket {
  pub def: &'static PocketDef,
  pub stacks: ItemStacks,
  pub volume_used: Volume,
  pub weight: Mass,
  pub priority: i32,
  pub closed: bool,
}

impl PartialEq for Pocket {
  fn eq(&self, other: &Self) -> bool { self.def == other.def }
}
impl Deref for Pocket {
  type Target = PocketDef;

  fn deref(&self) -> &Self::Target { self.def }
}
impl From<&'static PocketDef> for Pocket {
  fn from(value: &'static PocketDef) -> Self {
    Self {
      def: value,
      stacks: ItemStacks::new(),
      volume_used: Volume::ZERO,
      weight: Mass::ZERO,
      priority: 0,
      closed: true,
    }
  }
}
impl Pocket {
  pub fn recalc_volume_and_size(&mut self) {
    self.volume_used = self.stacks.iter().fold(Volume::ZERO, |v, i| v + i.volume());
    self.weight = self.stacks.iter().fold(Mass::ZERO, |v, i| v + i.weight());
  }

  pub fn holdable_count(&self, stack: &ItemStack) -> u64 {
    if self.closed
      || (!self.def.can_store_liquid && stack.item.phase == Phase::Liquid)
      || self.def.specify_items.map(|f| f(&stack.item)) == Some(false)
    {
      return 0;
    }
    let volume_cap: u64 = ((self.capacity - self.volume_used) / stack.item.volume.into()).into();
    let max_hold = volume_cap.min(stack.count);
    if max_hold == 0 {
      return 0;
    }
    let weight_cap = ((self.max_weight - self.weight) / stack.item.weight.into()).into();
    max_hold.min(weight_cap)
  }

  pub fn insert_stack_from(&mut self, stack: &mut ItemStack) {
    let count = self.holdable_count(stack);
    if count == 0 {
      return;
    }
    self.volume_used += stack.item.volume * count;
    self.weight += stack.item.weight * count;
    stack.count -= count;
    self.stacks.insert_stack(ItemStack { item: stack.item.clone(), count });
  }

  pub fn insert_stack(&mut self, mut stack: ItemStack) -> Option<ItemStack> {
    self.insert_stack_from(&mut stack);
    if stack.count == 0 { None } else { Some(stack) }
  }

  pub fn insert_stacks_from(&mut self, stacks: &mut ItemStacks) {
    for stack in 0..stacks.len() {
      self.insert_stack_from(&mut stacks[stack]);
      if stacks[stack].count == 0 {
        stacks.swap_remove(stack);
      }
    }
  }

  pub fn insert_stacks(&mut self, mut stacks: ItemStacks) -> Option<ItemStacks> {
    self.insert_stacks_from(&mut stacks);
    if stacks.is_empty() { None } else { Some(stacks) }
  }

  pub fn remove_items_matching<F: Fn(&Item) -> bool>(
    &mut self, f: F, count: u64,
  ) -> (ItemStacks, u64) {
    let (removed, mismatch) = self.stacks.remove_items_matching(f, count);
    for is in &removed.0 {
      self.volume_used -= is.volume();
      self.weight -= is.weight();
    }
    (removed, mismatch)
  }

  pub fn remove_items(&mut self, item: &Item, count: u64) {
    self.remove_items_matching(|ei| ei == item, count);
  }
}

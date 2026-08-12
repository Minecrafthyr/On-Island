use super::*;
#[derive(PartialEq)]
pub struct PocketDef {
  pub id: &'static str,
  pub capacity: Volume,
  pub max_weight: Mass,
  pub rigid: bool,
}
#[derive(Clone, PartialEq)]
pub struct Pocket {
  pub def: &'static PocketDef,
  pub item_stacks: ItemStacks,
  pub volume_used: Volume,
  pub weight: Mass,
}
impl Deref for Pocket {
  type Target = PocketDef;

  fn deref(&self) -> &Self::Target { self.def }
}
impl Pocket {
  pub fn holdable_count(&self, items: &ItemStack) -> u64 {
    let volume_cap: u64 = ((self.capacity - self.volume_used) / items.item.volume.into()).into();
    let max_hold = volume_cap.min(items.count);
    if max_hold == 0 {
      return 0;
    }
    let weight_cap = ((self.max_weight - self.weight) / items.item.weight.into()).into();
    max_hold.min(weight_cap)
  }

  pub fn insert_items_from(&mut self, items: &mut ItemStack) {
    let count = self.holdable_count(&items);
    if count == 0 {
      return;
    }
    self.volume_used += items.item.volume * count;
    self.weight += items.item.weight * count;
    items.count -= count;
    self.item_stacks.insert_items(ItemStack { item: items.item.clone(), count });
  }

  pub fn insert_items(&mut self, mut items: ItemStack) -> Option<ItemStack> {
    self.insert_items_from(&mut items);
    if items.count == 0 { None } else { Some(items) }
  }

  pub fn insert_stacks_from(&mut self, stacks: &mut ItemStacks) {
    for stack in 0..stacks.len() {
      self.insert_items_from(&mut stacks[stack]);
      if stacks[stack].count == 0 {
        stacks.swap_remove(stack);
      }
    }
  }

  pub fn insert_stacks(&mut self, mut stacks: ItemStacks) -> Option<ItemStacks> {
    self.insert_stacks_from(&mut stacks);
    if stacks.is_empty() { None } else { Some(stacks) }
  }
}
#[derive(PartialEq, Clone)]
pub struct ContainerDef {
  pub pocket_defs: &'static [PocketDef],
}
#[derive(Clone, PartialEq)]
pub struct Container {
  pub def: &'static ContainerDef,
  pub pockets: Vec<Pocket>,
}
impl Deref for Container {
  type Target = ContainerDef;

  fn deref(&self) -> &Self::Target { self.def }
}
const impl From<&'static ContainerDef> for Container {
  fn from(value: &'static ContainerDef) -> Self { Self { def: value, pockets: Vec::new() } }
}
impl Container {
  pub fn insert_items_from(&mut self, items: &mut ItemStack) {
    while let Some(pocket) = self.pockets.iter_mut().find(|pocket| pocket.holdable_count(items) > 0)
    {
      pocket.insert_items_from(items);
    }
  }

  pub fn insert_stacks_from(&mut self, stacks: &mut ItemStacks) {
    for stack in stacks.iter_mut() {
      self.insert_items_from(stack);
    }
    for i in (0..stacks.len()).rev() {
      if stacks[i].count == 0 {
        stacks.swap_remove(i);
      }
    }
  }
}

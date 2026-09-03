use std::mem::transmute;

use super::*;
#[derive(Clone, Copy)]
#[derive_const(Default)]
pub struct ItemDefStacks(pub &'static [ItemDefStack]);
const impl Deref for ItemDefStacks {
  type Target = &'static [ItemDefStack];

  fn deref(&self) -> &Self::Target { &self.0 }
}
const impl DerefMut for ItemDefStacks {
  fn deref_mut(&mut self) -> &mut Self::Target { &mut self.0 }
}

const impl From<&'static [ItemDefStack]> for ItemDefStacks {
  fn from(value: &'static [ItemDefStack]) -> Self { Self(value) }
}
const impl<const S: usize> From<&'static [(&'static ItemDef, u64); S]> for ItemDefStacks {
  fn from(value: &'static [(&'static ItemDef, u64); S]) -> Self {
    Self(unsafe { transmute((value, S as u64)) })
  }
}
const impl<const S: usize> From<[(&'static ItemDef, u64); S]> for ItemDefStacks {
  fn from(value: [(&'static ItemDef, u64); S]) -> Self { Self(unsafe { transmute((&value, S)) }) }
}
#[derive(Clone, PartialEq)]
pub struct ItemStacks(pub Vec<ItemStack>);
const impl Deref for ItemStacks {
  type Target = Vec<ItemStack>;

  fn deref(&self) -> &Self::Target { &self.0 }
}
const impl DerefMut for ItemStacks {
  fn deref_mut(&mut self) -> &mut Self::Target { &mut self.0 }
}

const impl Default for ItemStacks {
  fn default() -> Self { Self::new() }
}
impl ItemStacks {
  pub fn iter(&self) -> impl Iterator<Item = &ItemStack> { self.0.iter() }

  pub const fn new() -> Self { Self(Vec::new()) }

  pub fn insert_item(&mut self, item: Item) { self.insert_stack(ItemStack { item, count: 1 }); }

  pub fn insert_stack(&mut self, stack: ItemStack) {
    if let Some(f) = self.iter_mut().find(|ei| ei.item == stack.item) {
      f.count += stack.count;
    } else {
      self.push(stack);
    }
  }

  pub fn insert_stacks(&mut self, stacks: ItemStacks) {
    for stack in stacks.0 {
      self.insert_stack(stack);
    }
  }

  pub fn count_of_matching<F: Fn(&Item) -> bool>(&self, f: F) -> u64 {
    self.iter().filter(|ei| f(&ei.item)).map(|stack| stack.count).sum()
  }

  pub fn count_of(&self, item: &Item) -> u64 { self.count_of_matching(|ei| ei == item) }

  pub fn remove_items(&mut self, item: &Item, count: u64) {
    self.remove_items_matching(|ei| ei == item, count);
  }

  pub fn remove_items_matching<F: Fn(&Item) -> bool>(
    &mut self, f: F, mut count: u64,
  ) -> (ItemStacks, u64) {
    let mut removed = ItemStacks::new();
    while let Some(i) = self.iter_mut().position(|ei| f(&ei.item)) {
      if count > self[i].count {
        count -= self[i].count;
        removed.insert_stack(self.remove(i));
      } else {
        if count < self[i].count {
          self[i].count = self[i].count.saturating_sub(count);
        } else {
          removed.insert_stack(self.remove(i));
        }
        return (removed, 0);
      }
    }
    (removed, count)
  }
}

impl<I, C: IntoIterator<Item = I>> From<C> for ItemStacks
where
  ItemStack: From<I>,
{
  fn from(value: C) -> Self { Self::from_iter(value) }
}
impl<I> FromIterator<I> for ItemStacks
where
  ItemStack: From<I>,
{
  fn from_iter<T: IntoIterator<Item = I>>(iter: T) -> Self {
    Self(iter.into_iter().map(ItemStack::from).collect())
  }
}

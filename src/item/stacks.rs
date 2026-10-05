use std::ops::{Index, IndexMut};

use super::*;
use crate::utils::CountOf;
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
const impl<const S: usize> From<&'static [ItemDefStack; S]> for ItemDefStacks {
  fn from(value: &'static [ItemDefStack; S]) -> Self { Self(value) }
}

#[derive(Clone, PartialEq)]
pub struct ItemStacks(pub Vec<ItemStack>);

impl Index<usize> for ItemStacks {
  type Output = ItemStack;

  fn index(&self, index: usize) -> &Self::Output { &self.0[index] }
}
impl IndexMut<usize> for ItemStacks {
  fn index_mut(&mut self, index: usize) -> &mut Self::Output { &mut self.0[index] }
}
impl Index<&Item> for ItemStacks {
  type Output = ItemStack;

  fn index(&self, index: &Item) -> &Self::Output {
    self.0.iter().find(|i| &i.item == index).unwrap()
  }
}
impl IndexMut<&Item> for ItemStacks {
  fn index_mut(&mut self, index: &Item) -> &mut Self::Output {
    self.0.iter_mut().find(|i| &i.item == index).unwrap()
  }
}

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

  pub fn insert_stack_from(&mut self, item: &Item, count: u64) {
    if let Some(f) = self.iter_mut().find(|ei| &ei.item == item) {
      f.count += count;
    } else {
      self.push(item.clone() * count);
    }
  }

  pub fn insert_stacks(&mut self, stacks: ItemStacks) {
    for stack in stacks.0 {
      self.insert_stack(stack);
    }
  }

  pub fn remove_items(&mut self, item: &Item, count: u64) {
    self.take_items_matching(|ei| ei == item, count);
  }

  pub fn take_items_matching<F: Fn(&Item) -> bool>(
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
impl<F: Fn(&Item) -> bool> CountOf<F> for ItemStacks {
  fn count_of(&self, f: F) -> u64 {
    self.iter().filter(|ei| f(&ei.item)).map(|stack| stack.count).sum()
  }
}
impl CountOf<&Item> for ItemStacks {
  fn count_of(&self, item: &Item) -> u64 { self.count_of(|ei: &Item| ei == item) }
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

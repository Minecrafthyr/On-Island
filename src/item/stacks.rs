use std::ops::{Index, IndexMut};

use inew::New;

use super::*;
use crate::utils::{IterStack, TakeItems};
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
#[derive_const(Default, New)]
#[new(const = true)]
pub struct ItemStacks(#[new(default = vec![])] pub Vec<ItemStack>);
impl IntoIterator for ItemStacks {
  type IntoIter = <Vec<stack::ItemStack> as IntoIterator>::IntoIter;
  type Item = ItemStack;

  fn into_iter(self) -> Self::IntoIter { self.0.into_iter() }
}
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

impl ItemStacks {
  pub fn iter(&self) -> impl Iterator<Item = &ItemStack> { self.0.iter() }

  pub fn insert_from_defs(&mut self, defs: ItemDefStacks) {
    for def in defs.into_iter() {
      self.insert_stack((*def).into());
    }
  }

  pub fn insert_item(&mut self, item: Item) { self.insert_stack(ItemStack { item, count: 1 }); }

  pub fn insert_stack(&mut self, stack: ItemStack) {
    if let Some(f) = self.iter_mut().find(|ei| ei.item == stack.item) {
      f.count += stack.count;
    } else {
      self.push(stack);
    }
  }

  pub fn insert_items(&mut self, item: &Item, count: u64) {
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
}
impl IterStack for ItemStacks {
  fn iter_stack(&self) -> impl Iterator<Item = &ItemStack> { self.iter() }
}
impl TakeItems<&Item> for ItemStacks {
  fn take_items(&mut self, item: &Item, count: u64) -> (ItemStacks, u64) {
    self.take_items(|ei: &Item| ei == item, count)
  }
}
impl<F: Fn(&Item) -> bool> TakeItems<F> for ItemStacks {
  fn take_items(&mut self, f: F, mut count: u64) -> (ItemStacks, u64) {
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

impl<I: Into<ItemStack>> FromIterator<I> for ItemStacks {
  fn from_iter<T: IntoIterator<Item = I>>(iter: T) -> Self {
    Self(iter.into_iter().map(|i| i.into()).collect())
  }
}

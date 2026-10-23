use std::ops::{AddAssign, Index, IndexMut};

use enumflags2::BitFlags;
use inew::New;

use super::*;
use crate::{
  location::EnvData,
  utils::{IterStack, TakeItems},
};
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
  type IntoIter = <Vec<ItemStack> as IntoIterator>::IntoIter;
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

  pub fn tick(&mut self, env: BitFlags<EnvData>) -> ItemStacks {
    let mut converted = ItemStacks::new();
    for stack in &mut self.0 {
      if let Some(outputs) = stack.item.tick(env) {
        for mut output in outputs {
          output.count = output.count.saturating_mul(stack.count);
          converted += output;
        }
        stack.count = 0;
      }
    }
    self.0.retain(|stack| stack.count != 0);
    converted
  }

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
impl AddAssign<ItemDefStacks> for ItemStacks {
  fn add_assign(&mut self, rhs: ItemDefStacks) { self.insert_from_defs(rhs); }
}
impl AddAssign<Item> for ItemStacks {
  fn add_assign(&mut self, rhs: Item) { self.insert_item(rhs); }
}
impl AddAssign<(&Item, u64)> for ItemStacks {
  fn add_assign(&mut self, rhs: (&Item, u64)) { self.insert_items(rhs.0, rhs.1); }
}
impl AddAssign<ItemStack> for ItemStacks {
  fn add_assign(&mut self, rhs: ItemStack) { self.insert_stack(rhs); }
}
impl AddAssign<ItemStacks> for ItemStacks {
  fn add_assign(&mut self, rhs: ItemStacks) { self.insert_stacks(rhs); }
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
        removed += self.remove(i);
      } else {
        if count < self[i].count {
          self[i].count = self[i].count.saturating_sub(count);
        } else {
          removed += self.remove(i);
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

#[cfg(test)]
mod tests {
  use super::*;
  use crate::location::EnvData;

  #[test]
  fn conversion_replaces_each_item_in_a_stack() {
    let mut source = ItemStacks(vec![crate::item::LIVING_FISH.item() * 3]);
    let mut converted = ItemStacks::new();
    let env = EnvData::NaturalAir.into();

    for _ in 0..600_000 {
      converted += source.tick(env);
      if source.is_empty() {
        break;
      }
    }

    assert!(source.is_empty());
    assert_eq!(converted.len(), 1);
    assert!(converted[0].item.def == crate::item::RAW_FISH);
    assert_eq!(converted[0].count, 3);
  }
}

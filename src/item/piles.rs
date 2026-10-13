use std::ops::{Deref, DerefMut};

use derive_more::From;
use inew::New;

use crate::item::ItemStacks;
#[derive_const(From)]
pub struct ItemPile {
  pub stacks: ItemStacks,
}

impl Deref for ItemPile {
  type Target = ItemStacks;

  fn deref(&self) -> &Self::Target { &self.stacks }
}
impl DerefMut for ItemPile {
  fn deref_mut(&mut self) -> &mut Self::Target { &mut self.stacks }
}
#[derive(Default, New)]
#[new(const = true)]
pub struct ItemPiles(pub Vec<ItemPile>);
impl Deref for ItemPiles {
  type Target = Vec<ItemPile>;

  fn deref(&self) -> &Self::Target { &self.0 }
}
impl DerefMut for ItemPiles {
  fn deref_mut(&mut self) -> &mut Self::Target { &mut self.0 }
}

impl<I, C: IntoIterator<Item = I>> From<C> for ItemPiles
where
  ItemPile: From<I>,
{
  fn from(value: C) -> Self { Self::from_iter(value) }
}
impl<I> FromIterator<I> for ItemPiles
where
  ItemPile: From<I>,
{
  fn from_iter<T: IntoIterator<Item = I>>(iter: T) -> Self {
    Self(iter.into_iter().map(ItemPile::from).collect())
  }
}
impl ItemPiles {
  pub fn iter(&self) -> impl Iterator<Item = &ItemPile> { self.0.iter() }
}

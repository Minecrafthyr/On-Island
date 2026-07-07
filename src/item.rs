use std::ops::{Deref, DerefMut};

use rand::RngExt;
use strum_macros::{EnumString, IntoStaticStr};
use time::Duration;

use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, EnumString, IntoStaticStr)]
#[strum(serialize_all = "snake_case")]
pub enum Item {
    Biscuit,
    Water,
    RawFish,
    Wood,
    Tree,
    Rock,
}
impl NameAndDesc for Item {
    const PREFIX: &str = "item";

    fn get_id(&self) -> &str { self.into() }
}

impl Item {
    pub fn pick_up_time(&self) -> Option<Duration> {
        use Item::*;
        match self {
        Biscuit | Water | Rock | RawFish => Some(Duration::milliseconds(500)),
        Wood => Some(Duration::seconds(5)),
        _ => None,
        }
    }

    pub fn gather_items(&self, game: &mut Game) -> Option<(Duration, f64, ItemStacks)> {
        match self {
        Item::RawFish => Some((
            Duration::seconds(game.rng.random_range(120..=2000)),
            1.2,
            [(Item::RawFish, 1)].into(),
        )),
        Item::Tree => Some((Duration::minutes(30), 2.0, [(Item::Wood, 2)].into())),
        _ => None,
        }
    }

    pub fn use_data(&self) -> Option<(Duration, f64, Vec<(Attribute, Duration)>)> {
        match self {
        Item::Biscuit => Some((Duration::seconds(5), 1.0, vec![
            (Energy, Duration::hours(1)),
            (Water, Duration::hours(-1)),
        ])),
        Item::Water => Some((Duration::seconds(5), 1.0, vec![(Water, Duration::hours(2))])),
        Item::RawFish => Some((Duration::seconds(60), 1.0, vec![
            (Energy, Duration::hours(2)),
            (Water, Duration::minutes(50)),
        ])),
        _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ItemStack {
    pub item: Item,
    pub count: u64,
}

impl ItemStack {
    pub fn new(item: Item, count: u64) -> Self { Self { item, count } }
}
impl From<(Item, u64)> for ItemStack {
    fn from(value: (Item, u64)) -> Self { Self { item: value.0, count: value.1 } }
}

#[derive(Debug, Clone)]
pub struct ItemStacks(pub Vec<ItemStack>);
impl Deref for ItemStacks {
    type Target = Vec<ItemStack>;

    fn deref(&self) -> &Self::Target { &self.0 }
}
impl DerefMut for ItemStacks {
    fn deref_mut(&mut self) -> &mut Self::Target { &mut self.0 }
}

impl Default for ItemStacks {
    fn default() -> Self { Self::new() }
}

impl ItemStacks {
    pub const fn new() -> Self { Self(Vec::new()) }

    pub fn insert(&mut self, item: Item) {
        self.iter_mut()
            .find(|ei| ei.item == item)
            .map(|stack| stack.count += 1)
            .unwrap_or_else(|| self.push(ItemStack { item, count: 1 }));
    }

    pub fn insert_stack(&mut self, item_stack: ItemStack) {
        self.iter_mut()
            .find(|ei| ei.item == item_stack.item)
            .map(|stack| stack.count += item_stack.count)
            .unwrap_or_else(|| self.push(item_stack));
    }

    pub fn remove_item(&mut self, item: Item, count: u64) {
        let Some(i) = self.iter_mut().position(|ei| ei.item == item) else { return };
        self.0[i].count -= count;
        if self.0[i].count == 0 {
            self.0.remove(i);
        }
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

use std::ops::{Deref, DerefMut};

use rand::RngExt;

use super::*;
use crate::time::offset::TimeOffset;
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Item {
    Biscuit,
    Water,
    RawFish,
    Wood,
    Rock,
}

impl Item {
    pub fn name(&self) -> &'static str {
        match self {
        Item::Biscuit => "饼干",
        Item::Water => "水",
        Item::RawFish => "鱼",
        Item::Wood => "木头",
        Item::Rock => "石块",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
        Item::Biscuit => "一些饼干，会让你口渴。",
        Item::Water => "清凉的水。",
        Item::RawFish => "新鲜的鱼，可以吃。",
        Item::Wood => "坚硬的木头。",
        Item::Rock => "坚硬的石块。",
        }
    }

    pub fn gather_time(&self, game: &mut Game) -> Time {
        match self {
        Item::Biscuit => Time::s(5),
        Item::Water => Time::s(5),
        Item::RawFish => Time::s(game.rng.random_range(120..=2000)),
        Item::Wood => Time::m(30),
        Item::Rock => Time::m(1),
        }
    }

    pub fn use_data(&self) -> Option<(Time, Vec<(Attribute, TimeOffset)>)> {
        match self {
        Item::Biscuit =>
            Some((Time::s(5), vec![(Energy, TimeOffset::h(1)), (Water, TimeOffset::h(-1))])),
        Item::Water => Some((Time::s(5), vec![(Water, TimeOffset::h(2))])),
        Item::RawFish =>
            Some((Time::s(60), vec![(Energy, TimeOffset::h(2)), (Water, TimeOffset::m(50))])),
        _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ItemStack {
    pub item: Item,
    pub count: u64,
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
impl From<Vec<ItemStack>> for ItemStacks {
    fn from(value: Vec<ItemStack>) -> Self { Self(value) }
}

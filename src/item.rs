use std::{
    fmt::Display,
    hash::Hash,
    mem::transmute,
    ops::{Deref, DerefMut},
};

use time::Duration;

use super::*;
use crate::units::*;
define!(
    #[derive(Debug)]
    pub struct ItemDef {
        pub id: &'static str,
        pub volume: Volume,
        pub weight: Mass,
        pub container_size: Volume,
    },
    [
        ("biscuit").volume(mL(100)).weight(g(20)),
        ("water").volume(mL(501)).weight(g(502)),
        ("raw_fish").volume(L(1)).weight(g(1200)),
        ("wood").volume(L(50)).weight(kg(20)),
        ("tree").volume(L(150)).weight(kg(100)),
        ("stick").volume(L(1)).weight(g(800)),
        ("rock").volume(L(150)).weight(kg(100)),
        ("tree_vine").volume(mL(500)).weight(g(300)),
        ("dry_tree_vine").volume(mL(400)).weight(g(250)),
        ("vine_backpack").volume(L(20)).weight(kg(3)),
        ("vine_basket").volume(L(30)).weight(kg(4)),
    ]
);

impl NameAndDesc for ItemDef {
    const PREFIX: &str = "item";

    fn get_id(&self) -> &str { self.id }
}

#[derive(Debug, Clone, Copy, PartialEq, Hash)]
pub struct Item {
    pub def: &'static ItemDef,
}

impl Item {
    pub const fn new(def: &'static ItemDef) -> Self { Self { def } }

    pub fn use_data(&self) -> Option<(Duration, f64, Vec<(Attribute, Duration)>)> {
        match self.get_id() {
        "biscuit" => Some((Duration::seconds(5), 1.0, vec![
            (Energy, Duration::hours(1)),
            (Water, Duration::hours(-1)),
        ])),
        "water" => Some((Duration::seconds(5), 1.0, vec![(Water, Duration::hours(2))])),
        "raw_fish" => Some((Duration::seconds(60), 1.0, vec![
            (Energy, Duration::hours(2)),
            (Water, Duration::minutes(50)),
        ])),
        _ => None,
        }
    }
}

impl Deref for Item {
    type Target = ItemDef;

    fn deref(&self) -> &Self::Target { self.def }
}
const impl From<&'static ItemDef> for Item {
    fn from(value: &'static ItemDef) -> Self { Self { def: value } }
}

#[derive(Debug, Clone, Copy, PartialEq, Hash)]
pub struct ItemStack {
    pub item: Item,
    pub count: u64,
}

impl ItemStack {
    pub const fn new(item: Item, count: u64) -> Self { Self { item, count } }

    pub const fn from_def(item: &'static ItemDef, count: u64) -> Self {
        Self { item: item.into(), count: count }
    }
}
const impl From<(Item, u64)> for ItemStack {
    fn from(value: (Item, u64)) -> Self { Self::new(value.0, value.1) }
}
const impl From<ItemDefStack> for ItemStack {
    fn from(value: ItemDefStack) -> Self {
        Self { item: Item::new(value.item), count: value.count }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ItemDefStack {
    pub item: &'static ItemDef,
    pub count: u64,
}

impl ItemDefStack {
    pub const fn new(item: &'static ItemDef, count: u64) -> Self { Self { item, count } }
}
const impl From<(&'static ItemDef, u64)> for ItemDefStack {
    fn from(value: (&'static ItemDef, u64)) -> Self { Self { item: value.0, count: value.1 } }
}
impl Display for ItemDefStack {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} ×{}", Item::new(self.item).name(), self.count)
    }
}
#[derive(Debug, Clone, Copy)]
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
        Self(unsafe { transmute((value, S)) })
    }
}
const impl<const S: usize> From<[(&'static ItemDef, u64); S]> for ItemDefStacks {
    fn from(value: [(&'static ItemDef, u64); S]) -> Self { Self(unsafe { transmute((&value, S)) }) }
}

#[derive(Debug, Clone)]
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

    pub fn count_of(&self, item: Item) -> u64 {
        self.iter().find(|ei| ei.item == item).map_or(0, |stack| stack.count)
    }

    pub fn remove_item(&mut self, item: Item, count: u64) {
        let Some(i) = self.iter_mut().position(|ei| ei.item == item) else { return };
        self.0[i].count = self.0[i].count.saturating_sub(count);
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

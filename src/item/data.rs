use crate::{
  attribute::{AttributeValue::*, *},
  item::UseData,
  units::*,
};
pub struct ItemDef {
  pub id: &'static str,
  pub volume: Volume,
  pub weight: Mass,
  pub container_size: Volume,
  pub use_data: Option<UseData>,
}
impl ItemDef {
  pub const fn new(id: &'static str, volume: Volume, weight: Mass) -> Self {
    Self { id, volume, weight, container_size: Default::default(), use_data: None }
  }

  pub const fn container_size(mut self, value: Volume) -> Self {
    self.container_size = value;
    self
  }

  pub const fn use_data(mut self, use_data: UseData) -> Self {
    self.use_data = Some(use_data);
    self
  }
}
impl PartialEq for ItemDef {
  fn eq(&self, other: &Self) -> bool { self.id == other.id }
}
impl std::hash::Hash for ItemDef {
  fn hash<H: std::hash::Hasher>(&self, state: &mut H) { self.id.hash(state); }
}
pub const BISCUIT: &ItemDef = &ItemDef::new("biscuit", mL(100), g(20)).use_data(UseData {
  dur: seconds(5),
  activity: 1.03,
  attrs: &[(Energy, Dur(hours(1))).into(), (Water, Dur(hours(-1))).into()],
});
pub const BERRY: &ItemDef = &ItemDef::new("berry", mL(6), g(3)).use_data(UseData {
  dur: milliseconds(100),
  activity: 1.02,
  attrs: &[(Energy, Dur(minutes(2))).into(), (Water, Dur(minutes(1))).into()],
});
pub const BERRY_BRANCH: &ItemDef = &ItemDef::new("berry_branch", mL(300), g(200));
pub const BERRY_BUSH: &ItemDef = &ItemDef::new("berry_bush", L(6), kg(2));
pub const WATER_BOTTLE: &ItemDef = &ItemDef::new("water_bottle", mL(501), g(502))
  .use_data(UseData { dur: seconds(5), activity: 1.03, attrs: &[(Water, Dur(hours(2))).into()] });
pub const RAW_FISH: &ItemDef = &ItemDef::new("raw_fish", L(1), g(1200)).use_data(UseData {
  dur: seconds(60),
  activity: 1.06,
  attrs: &[(Energy, Dur(hours(2))).into(), (Water, Dur(minutes(50))).into()],
});
pub const WOOD: &ItemDef = &ItemDef::new("wood", L(50), kg(20));
pub const TREE: &ItemDef = &ItemDef::new("tree", L(150), kg(100));
pub const STICK: &ItemDef = &ItemDef::new("stick", L(1), g(800));
pub const ROCK: &ItemDef = &ItemDef::new("rock", L(150), kg(100));
pub const TREE_VINE: &ItemDef = &ItemDef::new("tree_vine", mL(500), g(300));
pub const DRY_TREE_VINE: &ItemDef = &ItemDef::new("dry_tree_vine", mL(400), g(200));
pub const VINE_BACKPACK: &ItemDef =
  &ItemDef::new("vine_backpack", L(25), kg(1)).container_size(L(18));
pub const VINE_BASKET: &ItemDef = &ItemDef::new("vine_basket", L(50), kg(2)).container_size(L(48));
pub const CANVAS_BACKPACK: &ItemDef =
  &ItemDef::new("canvas_backpack", L(28), g(700)).container_size(L(20));
pub const COTTON_UNDERWEAR: &ItemDef =
  &ItemDef::new("cotton_underwear", L(5), g(1200)).container_size(mL(500));
pub const COTTON_PANTIES: &ItemDef = &ItemDef::new("cotton_panties", L(1), g(200));
pub const ITEMDEFS: &[&ItemDef] = &[
  BISCUIT,
  WATER_BOTTLE,
  RAW_FISH,
  WOOD,
  TREE,
  STICK,
  ROCK,
  TREE_VINE,
  DRY_TREE_VINE,
  VINE_BACKPACK,
  VINE_BASKET,
  CANVAS_BACKPACK,
  COTTON_UNDERWEAR,
  COTTON_PANTIES,
];
pub const ITEMDEFS_MAP: ::phf::Map<&'static str, &'static ItemDef> = ::phf::phf_map! {
    "biscuit" =>  &BISCUIT,"water" =>  &WATER_BOTTLE,"raw_fish" =>  &RAW_FISH,"wood" =>  &WOOD,"tree" =>  &TREE,"stick" =>  &STICK,"rock" =>  &ROCK,"tree_vine" =>  &TREE_VINE,"dry_tree_vine" =>  &DRY_TREE_VINE,"vine_backpack" =>  &VINE_BACKPACK,"vine_basket" =>  &VINE_BASKET,"canvas_backpack" =>  &CANVAS_BACKPACK,"cotton_underwear" =>  &COTTON_UNDERWEAR,"cotton_panties" =>  &COTTON_PANTIES,
};

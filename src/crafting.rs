use itertools::Itertools;
use time::Duration;

use crate::item::{Item, ItemDefStack, ItemDefStacks, ItemStacks};

#[derive(Debug, Clone, Copy)]
pub struct CraftingRecipe {
  pub id: &'static str,
  pub name_key: &'static str,
  pub inputs: ItemDefStacks,
  pub outputs: ItemDefStacks,
  pub time: Duration,
  pub activity: f64,
}

impl CraftingRecipe {
  pub fn name(&self) -> String { t!(self.name_key).into_owned() }

  pub fn inputs_text(&self) -> String {
    self.inputs.iter().map(ItemDefStack::to_string).join(" + ")
  }

  pub fn outputs_text(&self) -> String {
    self.outputs.iter().map(ItemDefStack::to_string).join(" + ")
  }

  pub fn can_apply(&self, inventory: &ItemStacks) -> bool {
    self.inputs.iter().all(|ids| inventory.count_of(Item::new(ids.item)) >= ids.count)
  }

  pub fn max_batch_count(&self, inventory: &ItemStacks) -> u64 {
    self
      .inputs
      .iter()
      .map(|ids| {
        let available = inventory.count_of(Item::new(ids.item));
        available / ids.count
      })
      .min()
      .unwrap_or(0)
  }
}
use crate::item::*;

pub const CRAFTING_RECIPES: &[CraftingRecipe] = &[
  CraftingRecipe {
    id: "dry_tree_vine",
    name_key: "crafting.dry_tree_vine.name",
    inputs: (&[(TREE_VINE, 1)]).into(),
    outputs: (&[(DRY_TREE_VINE, 1)]).into(),
    time: Duration::hours(1),
    activity: 1.4,
  },
  CraftingRecipe {
    id: "vine_backpack",
    name_key: "crafting.vine_backpack.name",
    inputs: (&[(DRY_TREE_VINE, 5)]).into(),
    outputs: (&[(VINE_BACKPACK, 1)]).into(),
    time: Duration::hours(2),
    activity: 1.6,
  },
  CraftingRecipe {
    id: "vine_basket",
    name_key: "crafting.vine_basket.name",
    inputs: (&[(DRY_TREE_VINE, 10)]).into(),
    outputs: (&[(VINE_BASKET, 1)]).into(),
    time: Duration::hours(3),
    activity: 1.6,
  },
];

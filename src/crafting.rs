use itertools::Itertools;
use time::Duration;

use crate::item::*;

#[derive(Clone, Copy)]
pub struct Recipe {
  pub id: &'static str,
  pub name_key: &'static str,
  pub inputs: ItemDefStacks,
  pub outputs: ItemDefStacks,
  pub time: Duration,
  pub activity: f64,
}

impl Recipe {
  pub fn name(&self) -> String { t!(self.name_key).into_owned() }

  pub fn inputs_text(&self) -> String { self.inputs.iter().map(ToString::to_string).join(" + ") }

  pub fn outputs_text(&self) -> String { self.outputs.iter().map(ToString::to_string).join(" + ") }
}

pub enum RecipeComponents {
  AnyOf(&'static [RecipeComponents]),
  Value { stack: ItemDefStack, apply_for_each: bool },
}

pub const CRAFTING_RECIPES: &[Recipe] = &[
  Recipe {
    id: "dry_tree_vine",
    name_key: "crafting.dry_tree_vine.name",
    inputs: (&[(TREE_VINE, 1)]).into(),
    outputs: (&[(DRY_TREE_VINE, 1)]).into(),
    time: Duration::hours(1),
    activity: 1.3,
  },
  Recipe {
    id: "vine_backpack",
    name_key: "crafting.vine_backpack.name",
    inputs: (&[(DRY_TREE_VINE, 5)]).into(),
    outputs: (&[(VINE_BACKPACK, 1)]).into(),
    time: Duration::hours(2),
    activity: 1.4,
  },
  Recipe {
    id: "vine_basket",
    name_key: "crafting.vine_basket.name",
    inputs: (&[(DRY_TREE_VINE, 10)]).into(),
    outputs: (&[(VINE_BASKET, 1)]).into(),
    time: Duration::hours(3),
    activity: 1.4,
  },
];

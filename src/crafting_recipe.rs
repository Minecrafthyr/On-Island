use std::{borrow::Cow, fmt::Write};

use time::Duration;

use crate::item::*;

#[derive(Clone)]
pub struct RecipeStep {
  pub custom_name: Option<&'static str>,
  pub inputs: RecipeDepends,
  pub outputs: ItemDefStacks,
  pub time: Duration,
  pub activity: f64,
}
#[derive(Clone)]
pub struct Recipe {
  pub id: &'static str,
  pub custom_name: Option<&'static str>,
  pub steps: &'static [RecipeStep],
}

impl Recipe {
  pub fn name(&self) -> Cow<'_, str> {
    if let Some(custom_name) = self.custom_name {
      t!(format!("crafting.{}.name", custom_name))
    } else {
      t!(format!("crafting.{}.name", self.id))
    }
  }

  // pub fn depends(&self) -> Vec<&'static RecipeStep> { self.steps.iter() }

  pub fn io_text(&self) -> String {
    let mut r = String::new();
    for (step, rs) in self.steps.iter().enumerate() {
      let ik = format!("crafting.step{}", step);
      let tik = t!(ik);
      let _ = writeln!(
        r,
        "{}",
        t!(
          "crafting_recipe.step",
          index = step,
          depends = if tik != ik { tik } else { Cow::Borrowed("???") },
          time = rs.time,
          activity = rs.activity: {:.2}
        )
      );
    }
    r
  }
}
#[derive(Clone)]
pub enum ItemRequirements {
  Def(&'static ItemDef),
  Defs(&'static [&'static ItemDef]),
  Fn(&'static dyn Fn(&Item) -> bool),
}
use ItemRequirements as ItemReq;
impl ItemReq {
  pub fn matches(&self, i: &Item) -> bool {
    match *self {
    ItemReq::Def(item_def) => i.def == item_def,
    ItemReq::Defs(item_defs) => item_defs.contains(&i.def),
    ItemReq::Fn(f) => f(i),
    }
  }
}
const impl From<&'static dyn Fn(&Item) -> bool> for ItemReq {
  fn from(value: &'static dyn Fn(&Item) -> bool) -> Self { Self::Fn(value) }
}
const impl From<&'static ItemDef> for ItemReq {
  fn from(value: &'static ItemDef) -> Self { Self::Def(value) }
}
const impl From<&'static [&'static ItemDef]> for ItemReq {
  fn from(value: &'static [&'static ItemDef]) -> Self { Self::Defs(value) }
}
#[derive(Clone)]
pub struct RecipeRequirements {
  pub item: ItemReq,
  pub count: u64,
  pub consume: bool,
  pub one_by_one: bool,
}
use RecipeRequirements as Req;
impl Req {
  pub const fn comp(item: impl const Into<ItemReq>) -> Self {
    Self { item: item.into(), count: 1, one_by_one: true, consume: true }
  }

  pub const fn c(mut self, count: u64) -> Self {
    self.count = count;
    self
  }

  pub const fn no_consume(mut self) -> Self {
    self.consume = false;
    self
  }

  pub const fn consume_once(mut self) -> Self {
    self.one_by_one = false;
    self
  }
}
#[derive(Clone)]
pub enum RecipeDepends {
  Req(Req),
  AllOf(&'static [RecipeDepends]),
  AnyOf(&'static [RecipeDepends]),
}
impl RecipeDepends {
  pub const fn comp(item: impl const Into<ItemReq>) -> Self { Self::Req(Req::comp(item)) }
}
const impl From<Req> for RecipeDepends {
  fn from(value: Req) -> Self { Self::Req(value) }
}

pub const CRAFTING_RECIPES: &[Recipe] = &[
  Recipe {
    id: "dry_tree_vine",
    custom_name: None,
    steps: &[RecipeStep {
      custom_name: None,
      inputs: RecipeDepends::AllOf(&[
        Req::comp(TREE_VINE).into(),
        Req::comp(FIRE).c(100).no_consume().into(),
      ]),
      outputs: (&[(DRY_TREE_VINE, 1)]).into(),
      time: Duration::hours(1),
      activity: 1.3,
    }],
  },
  Recipe {
    id: "vine_backpack",
    custom_name: None,
    steps: &[RecipeStep {
      custom_name: None,
      inputs: Req::comp(DRY_TREE_VINE).c(5).into(),
      outputs: (&[(VINE_BACKPACK, 1)]).into(),
      time: Duration::hours(2),
      activity: 1.4,
    }],
  },
  Recipe {
    id: "vine_basket",
    custom_name: None,
    steps: &[RecipeStep {
      custom_name: None,
      inputs: Req::comp(DRY_TREE_VINE).c(10).into(),
      outputs: (&[(VINE_BASKET, 1)]).into(),
      time: Duration::hours(3),
      activity: 1.4,
    }],
  },
];

use std::fmt::Write;

use crate::{item::*, preclude::*};
pub mod depends;
use depends::*;

pub const CRAFTING_RECIPES: &[Recipe] = &[
  Recipe::new("dry_tree_vine", &[Step::new(
    Depends::AllOf(&[
      TREE_VINE.into(),
      Req::from(FIRE * 100).no_consume().into(),
      (FIRE * 25).into(),
    ]),
    &[(DRY_TREE_VINE * 1)],
    1.h(),
    1.3,
  )]),
  Recipe::new("vine_backpack", &[Step::new(DRY_TREE_VINE * 5, &[VINE_BACKPACK * 1], 2.h(), 1.4)]),
  Recipe::new("vine_basket", &[Step::new(DRY_TREE_VINE * 10, &[VINE_BASKET * 1], 3.h(), 1.4)]),
];

#[derive(Clone)]
pub struct Recipe {
  pub id: &'static str,
  pub custom_name: Option<&'static str>,
  pub steps: &'static [Step],
}
#[derive(Clone)]
pub struct RecipeStep {
  pub custom_name: Option<&'static str>,
  pub inputs: Depends,
  pub outputs: ItemDefStacks,
  pub time: Duration,
  pub activity: f64,
}
use RecipeStep as Step;

impl Step {
  pub const fn new(
    inputs: impl const Into<Depends>, outputs: impl const Into<ItemDefStacks>, time: Duration,
    activity: f64,
  ) -> Self {
    Self { custom_name: None, inputs: inputs.into(), outputs: outputs.into(), time, activity }
  }
}
impl NameAndDesc for Recipe {
  const PREFIX: &str = "crafting.recipe";

  fn get_id(&self) -> Cow<'_, str> { Cow::Borrowed(self.id) }

  fn name(&self) -> Cow<'_, str> {
    if let Some(n) = self.custom_name {
      t!(n)
    } else {
      if let s = format!("{}.{}.name", Self::PREFIX, self.get_id())
        && let r = t!(s.clone())
        && r != s
      {
        r
      } else {
        t!("crafting.recipe")
      }
    }
  }

  fn description(&self) -> Cow<'_, str> {
    let mut r = String::new();
    for (index, step) in self.steps.iter().enumerate() {
      let ik = format!("crafting.recipe.{}.step.{}.name", self.id, index);
      let tik = t!(ik);
      let _ = writeln!(
        r,
        "{}",
        t!(
          "crafting.recipe.step",
          index = index,
          depends = if tik != ik { tik } else { Cow::Borrowed("???") },
          time = step.time,
          activity = step.activity: {:.2}
        )
      );
    }
    r.into()
  }
}
impl Recipe {
  pub const fn new(id: &'static str, steps: &'static [Step]) -> Self {
    Self { id, custom_name: None, steps }
  }
}

use itertools::Itertools;

use super::*;
use crate::builder_method;
pub const CRAFTING_RECIPES: &[Crafting] = &[
  Crafting::new("dry_tree_vine", (2, 0.01), &[Step::new(
    Depends::AllOf(&[
      TREE_VINE.into(),
      Req::from(FIRE * 1000).consume(ConsumeType::None).into(),
      (FIRE * 250).into(),
    ]),
    &[DRY_TREE_VINE * 1],
    1.h(),
    1.3,
  )]),
  Crafting::new("vine_backpack", (2, 0.95), &[Step::new(
    DRY_TREE_VINE * 5,
    &[VINE_BACKPACK * 1],
    2.h(),
    1.4,
  )]),
  Crafting::new("vine_basket", (1, 1.0), &[Step::new(
    DRY_TREE_VINE * 10,
    &[VINE_BASKET * 1],
    3.h(),
    1.4,
  )]),
  Crafting::new("kill_fish", (1, 1.0), &[Step::new(LIVING_FISH * 1, &[RAW_FISH * 1], 20.s(), 1.1)]),
  Crafting::new("grill_fish", (1, 1.0), &[Step::new(
    Depends::AllOf(&[
      ItemReq::All(&[RAW_FISH.into(), LIVING_FISH.into()]).into(),
      Req::from(FIRE * 1000).consume(ConsumeType::None).into(),
      (FIRE * 400).into(),
    ]),
    &[GRILLED_FISH * 1],
    20.mnt(),
    1.2,
  )]),
];

#[derive(Clone)]
pub struct Crafting {
  pub id: &'static str,
  pub custom_name: Option<&'static str>,
  pub batch: (usize, f64),
  pub steps: &'static [Step],
}
#[derive(Clone)]
pub struct Step {
  pub custom_name: Option<&'static str>,
  pub inputs: Depends,
  pub outputs: ItemDefStacks,
  pub time: Duration,
  pub activity: f64,
}

impl Step {
  pub const fn new(
    inputs: impl const Into<Depends>, outputs: impl const Into<ItemDefStacks>, time: Duration,
    activity: f64,
  ) -> Self {
    Self { custom_name: None, inputs: inputs.into(), outputs: outputs.into(), time, activity }
  }
}
impl NameAndDesc for Crafting {
  const PREFIX: &str = "crafting.recipe";

  fn get_id(&self) -> Cow<'_, str> { Cow::Borrowed(self.id) }

  fn name(&self) -> Cow<'_, str> {
    if let Some(n) = self.custom_name {
      t!(n)
    } else if let s = format!("{}.{}.name", Self::PREFIX, self.get_id())
      && let r = t!(s.clone())
      && r != s
    {
      r
    } else {
      let out = self
        .steps
        .iter()
        .filter_map(|s| {
          let v = s.outputs.iter().map(|o| o.item).collect_vec();
          if v.len() != 0 { Some(v.into_iter().map(|i| i.name()).join(", ")) } else { None }
        })
        .collect_vec();
      if out.len() != 0 { out.join("; ").into() } else { t!("crafting.recipe") }
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
impl Crafting {
  builder_method! {custom_name,Option<&'static str>}

  pub const fn new(id: &'static str, batch: (usize, f64), steps: &'static [Step]) -> Self {
    Self { id, custom_name: None, batch, steps }
  }
}

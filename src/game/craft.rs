use std::sync::Arc;

use itertools::Itertools;

use crate::{
  game::Game,
  io::ScreenWriter,
  item::{ItemStack, ItemStacks},
  player::{
    Effect, Player,
    action::{Action, ActionContent},
  },
  preclude::*,
  recipe::{
    crafting::{CRAFTING_RECIPES, Crafting, Step},
    depends::{ConsumeType, Depends, Req},
  },
  ui::{DataItem, DisplayList, NumberRequester},
  utils::{IterStack, TakeItems},
};
pub fn max_batch(c: &Vec<&ItemStack>, depends: &Depends) -> u64 {
  use Depends::*;
  match depends {
  AnyOf(items) => items.iter().fold(0u64, |b, d| b.max(max_batch(c, d))),
  AllOf(items) => items.iter().fold(0u64, |b, d| b.min(max_batch(c, d))),
  Req(req) => {
    let matched: u64 = c.iter().filter(|s| req.item.matches(&s.item)).map(|s| s.count).sum();
    if req.consume != ConsumeType::None { matched / req.count } else { u64::MAX }
  }
  }
}
pub fn max_batch_craft(c: &impl IterStack, craft: &Crafting) -> u64 {
  let mut new_items_pool = ItemStacks::new();
  let mut count = u64::MAX;
  for step in craft.steps {
    count = count
      .min(max_batch(&c.iter_stack().chain(new_items_pool.iter_stack()).collect(), &step.inputs));
    new_items_pool.insert_from_defs(step.outputs);
  }
  count
}

impl Game {
  pub fn craft(&mut self) -> Result<(), Box<dyn std::error::Error>> {
    let options: Vec<_> = CRAFTING_RECIPES.iter().collect();
    let mut s = ScreenWriter::new_screen();
    let Some(choice) = s.list(DisplayList {
      title: Some(t!("action.crafting.title")),
      data: options
        .iter()
        .enumerate()
        .map(|(i, recipe)| DataItem {
          text: t!("action.craft.entry", index = i, title = recipe.name()),
          selected: |o: &Cow<'_, str>| Cow::Owned(o.to_string() + &recipe.description()),
          enter: |i| Ok(i),
        })
        .collect(),
      ..
    })?
    else {
      return Ok(());
    };
    let recipe = options[choice];
    let Some(batch_count) = NumberRequester {
      prompt: t!("action.crafting.how_many"),
      range: Some(1..=max_batch(&self.player.iter_stack().collect(), &recipe.steps[0].inputs)), // TODO: fix this
      default: Some(1),
      ..
    }
    .request()?
    else {
      return Ok(());
    };
    for _ in 0..batch_count {
      // TODO: resume and one by one taking recipe req :)
      for rs in recipe.steps.iter() {
        match self.player_action(Action {
          id: "crafting",
          f: Arc::new(|_| ActionContent::new(vec![], vec![Effect::ActivityMul(rs.activity)])),
          total_dur: rs.time,
          ..
        }) {
        Ok(()) =>
          if !self.apply_recipe_step(rs) {
            s.message(t!("action.crafting.apply_failed", recipe = recipe.name()))?;
            return Ok(());
          },
        Err(not_ok) => {
          s.message(not_ok.to_string())?;
          return Ok(());
        }
        }
      }
    }
    Ok(())
  }

  pub fn apply_recipe_step(&mut self, recipe: &Step) -> bool {
    pub fn take_req(p: &mut Player, req: &Req) {
      if req.consume == ConsumeType::None {
        return;
      }
      p.take_items(|i| req.item.matches(i), req.count);
    }
    pub fn take_depends(p: &mut Player, depends: &Depends) {
      match depends {
      Depends::AnyOf(items) => take_depends(
        p,
        items
          .iter()
          .sorted_by(|l, r| {
            max_batch(&p.iter_stack().collect(), l).cmp(&max_batch(&p.iter_stack().collect(), r))
          }) // currently select reqs has highest max_batch_count.collect()
          .last()
          .unwrap(),
      ),
      Depends::AllOf(items) => items.iter().for_each(|i| take_depends(p, i)),
      Depends::Req(req) => take_req(p, req),
      }
    }
    take_depends(&mut self.player, &recipe.inputs);
    for ids in recipe.outputs.iter().copied() {
      if let Some(r) = self.player.insert_stack(ids.into()) {
        self.locations[self.player.location].pickup_stacks.insert_stack(r);
      }
    }
    true
  }
}

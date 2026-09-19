use std::borrow::Cow;

use itertools::Itertools;

use crate::{
  crafting_recipe::{CRAFTING_RECIPES, RecipeDepends, RecipeRequirements, RecipeStep},
  game::Game,
  io::ScreenWriter,
  player::{Action, ActionContent, Effect, Player},
  ui::{DataItem, DisplayList, NumberRequester},
};
pub fn test_req(p: &Player, req: &RecipeRequirements) -> u64 {
  use crate::crafting_recipe::ItemRequirements::*;
  let matched = match req.item {
  Def(item_def) => p.count_of_matching(|i| i.def == item_def),
  Defs(item_defs) => p.count_of_matching(|i| item_defs.contains(&i.def)),
  Fn(f) => p.count_of_matching(f),
  };
  if req.consume { matched / req.count } else { u64::MAX }
}
pub fn max_batch_count(p: &Player, depends: &RecipeDepends) -> u64 {
  use RecipeDepends::*;
  match depends {
  AnyOf(items) => items.iter().fold(0u64, |b, d| b.max(max_batch_count(p, d))),
  AllOf(items) => items.iter().fold(0u64, |b, d| b.min(max_batch_count(p, d))),
  Req(req) => test_req(p, req),
  }
}

impl Game {
  // TODO: we need pauseable craft and one by one taking recipe req :)
  pub fn craft(&mut self) {
    // pub fn test_depends(p: &Player, depends: &Depends) -> bool {
    //   match depends {
    //   Depends::AnyOf(items) => items.iter().any(|i| test_depends(p, i)),
    //   Depends::AllOf(items) => items.iter().all(|i| test_depends(p, i)),
    //   Depends::Req(req) => test_req(p, req) > 1,
    //   }
    // }
    let options: Vec<_> = CRAFTING_RECIPES.iter().collect();
    let mut s = ScreenWriter::new_screen();
    let Some(choice) = s.list(DisplayList::new(
      Some(t!("action.crafting.title")),
      options
        .iter()
        .enumerate()
        .map(|(i, recipe)| DataItem {
          text: t!("action.craft.entry", index = i, title = recipe.name()),
          selected: |o: &Cow<'_, str>| {
            let mut s = o.to_string();
            s.push_str(&recipe.io_text());
            Cow::Owned(s)
          },
          enter: |i| Ok(i),
        })
        .collect(),
    )) else {
      return;
    };
    let recipe = options[choice];
    let Some(batch_count) = NumberRequester::new(t!("action.crafting.how_many"))
      .range(1..=max_batch_count(&self.player, &recipe.steps[0].inputs)) // TODO: fix this
      .default(1)
      .request()
    else {
      return;
    };
    for _ in 0..batch_count {
      for rs in recipe.steps.iter() {
        match self.player_action(Action::no_progress(
          "crafting",
          |_| ActionContent { body_parts: vec![], effects: vec![Effect::ActivityMul(rs.activity)] },
          rs.time,
        )) {
        Ok(()) => {
          self.apply_recipe_step(rs);
        }
        Err(not_ok) => {
          s.message(not_ok.to_string());
          return;
        }
        }
      }
    }
  }

  pub fn apply_recipe_step(&mut self, recipe: &RecipeStep) -> bool {
    pub fn take_req(p: &mut Player, req: &RecipeRequirements) {
      if !req.consume {
        return;
      }
      use crate::crafting_recipe::ItemRequirements::*;
      match req.item {
      Def(item_def) => p.remove_items_matching(|i| i.def == item_def, req.count),
      Defs(item_defs) => p.remove_items_matching(|i| item_defs.contains(&i.def), req.count),
      Fn(f) => p.remove_items_matching(f, req.count),
      };
    }
    pub fn take_depends(p: &mut Player, depends: &RecipeDepends) {
      match depends {
      RecipeDepends::AnyOf(items) => take_depends(
        p,
        items
          .iter()
          .sorted_by(|l, r|
          // currently select reqs has max max_batch_count
          max_batch_count(p, l).cmp(&max_batch_count(p, r)))
          .last()
          .unwrap(),
      ),
      RecipeDepends::AllOf(items) => items.iter().for_each(|i| take_depends(p, i)),
      RecipeDepends::Req(req) => take_req(p, req),
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

use itertools::Itertools;

use crate::{
  game::Game,
  io::ScreenWriter,
  item::Item,
  player::{
    Effect, Player,
    action::{Action, ActionContent},
  },
  preclude::*,
  recipe::{
    crafting::{CRAFTING_RECIPES, Step},
    depends::{ConsumeType, Depends, Req},
  },
  ui::{DataItem, DisplayList, NumberRequester},
  utils::CountOf,
};

pub fn max_batch(p: &Player, depends: &Depends) -> u64 {
  use Depends::*;
  match depends {
  AnyOf(items) => items.iter().fold(0u64, |b, d| b.max(max_batch(p, d))),
  AllOf(items) => items.iter().fold(0u64, |b, d| b.min(max_batch(p, d))),
  Req(req) => {
    let matched = p.count_of(|i: &Item| req.item.matches(i));
    if req.consume != ConsumeType::None { matched / req.count } else { u64::MAX }
  }
  }
}

impl Game {
  pub fn craft(&mut self) {
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
            s.push_str(&recipe.description());
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
      .range(1..=max_batch(&self.player, &recipe.steps[0].inputs)) // TODO: fix this
      .default(1)
      .request()
    else {
      return;
    };
    for _ in 0..batch_count {
      // TODO: resume and one by one taking recipe req :)
      for rs in recipe.steps.iter() {
        match self.player_action(Action::no_progress(
          "crafting",
          |_| ActionContent::new(vec![], vec![Effect::ActivityMul(rs.activity)]),
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

  pub fn apply_recipe_step(&mut self, recipe: &Step) -> bool {
    pub fn take_req(p: &mut Player, req: &Req) {
      if req.consume == ConsumeType::None {
        return;
      }
      p.take_items_matching(|i| req.item.matches(i), req.count);
    }
    pub fn take_depends(p: &mut Player, depends: &Depends) {
      match depends {
      Depends::AnyOf(items) => take_depends(
        p,
        items
          .iter()
          .sorted_by(|l, r| max_batch(p, l).cmp(&max_batch(p, r))) // currently select reqs has highest max_batch_count
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

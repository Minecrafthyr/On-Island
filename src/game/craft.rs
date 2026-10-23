use std::{cmp::Reverse, sync::Arc};

use crate::{
  game::Game,
  io::ScreenWriter,
  item::{Item, ItemStack},
  player::{
    Effect, Player,
    action::{Action, ActionContent},
  },
  preclude::*,
  recipe::{
    crafting::{CRAFTING_RECIPES, Crafting, Step},
    depends::{ConsumeType, Depends},
  },
  ui::{DataItem, DisplayList, NumberRequester},
  utils::{IterStack, TakeItems},
};
pub fn max_batch(c: &Vec<&ItemStack>, depends: &Depends) -> u64 {
  use Depends::*;
  match depends {
  AnyOf(items) => items.iter().fold(0u64, |b, d| b.max(max_batch(c, d))),
  AllOf(items) => items.iter().fold(u64::MAX, |b, d| b.min(max_batch(c, d))),
  Req(req) => {
    let matched = c
      .iter()
      .filter(|s| req.item.matches(&s.item))
      .fold(0u64, |total, stack| total.saturating_add(stack.count));
    if matched < req.count {
      0
    } else if req.consume != ConsumeType::None && req.count != 0 {
      matched / req.count
    } else {
      u64::MAX
    }
  }
  }
}
pub fn max_batch_craft(c: &impl IterStack, craft: &Crafting) -> u64 {
  let mut inventory: Vec<_> =
    c.iter_stack().map(|stack| (stack.item.clone(), stack.count)).collect();
  let mut count = u64::MAX;
  for step in craft.steps {
    let stacks: Vec<_> = inventory.iter().map(|(item, count)| item.clone() * *count).collect();
    let available: Vec<_> = stacks.iter().collect();
    let mut upper = count.min(max_batch(&available, &step.inputs));
    let mut lower = 0;
    while lower < upper {
      let distance = upper - lower;
      let candidate = lower + distance / 2 + distance % 2;
      let mut candidate_inventory = inventory.clone();
      let mut candidate_consumed = Vec::new();
      if plan_dependency_consumption(
        &mut candidate_inventory,
        &step.inputs,
        &mut candidate_consumed,
        candidate,
      ) {
        lower = candidate;
      } else {
        upper = candidate - 1;
      }
    }
    count = lower;
    let mut consumed = Vec::new();
    if !plan_dependency_consumption(&mut inventory, &step.inputs, &mut consumed, count) {
      return 0;
    }
    for output in step.outputs.iter().copied() {
      let output_count = output.count.saturating_mul(count);
      if let Some((_, existing)) = inventory.iter_mut().find(|(item, _)| item.def == output.item) {
        *existing = existing.saturating_add(output_count);
      } else {
        inventory.push((Item::from(output.item), output_count));
      }
    }
  }
  count
}

fn plan_dependency_consumption(
  inventory: &mut Vec<(Item, u64)>, depends: &Depends, consumed: &mut Vec<(Item, u64)>,
  multiplier: u64,
) -> bool {
  match depends {
  Depends::Req(req) => {
    let available = inventory
      .iter()
      .filter(|(item, _)| req.item.matches(item))
      .map(|(_, count)| *count)
      .fold(0u64, |total, count| total.saturating_add(count));
    let needed_count = if req.consume == ConsumeType::None {
      req.count
    } else {
      req.count.saturating_mul(multiplier)
    };
    if available < needed_count {
      return false;
    }
    if req.consume == ConsumeType::None {
      return true;
    }
    let mut needed = needed_count;
    for (item, count) in inventory.iter_mut().filter(|(item, _)| req.item.matches(item)) {
      let take = (*count).min(needed);
      *count -= take;
      needed -= take;
      if take != 0 {
        if let Some((_, existing)) = consumed.iter_mut().find(|(existing, _)| existing == item) {
          *existing += take;
        } else {
          consumed.push((item.clone(), take));
        }
      }
      if needed == 0 {
        break;
      }
    }
    true
  }
  Depends::AllOf(items) =>
    items.iter().all(|item| plan_dependency_consumption(inventory, item, consumed, multiplier)),
  Depends::AnyOf(items) => {
    let stacks: Vec<_> = inventory
      .iter()
      .map(|(item, count)| ItemStack { item: item.clone(), count: *count })
      .collect();
    let available: Vec<_> = stacks.iter().collect();
    let mut alternatives: Vec<_> = items.iter().collect();
    alternatives.sort_by_key(|item| Reverse(max_batch(&available, item)));
    for item in alternatives {
      let mut candidate_inventory = inventory.clone();
      let mut candidate_consumed = consumed.clone();
      if plan_dependency_consumption(
        &mut candidate_inventory,
        item,
        &mut candidate_consumed,
        multiplier,
      ) {
        *inventory = candidate_inventory;
        *consumed = candidate_consumed;
        return true;
      }
    }
    false
  }
  }
}

fn plan_recipe_consumption(player: &Player, recipe: &Step) -> Option<Vec<(Item, u64)>> {
  let mut inventory: Vec<_> =
    player.iter_stack().map(|stack| (stack.item.clone(), stack.count)).collect();
  let mut consumed = Vec::new();
  plan_dependency_consumption(&mut inventory, &recipe.inputs, &mut consumed, 1).then_some(consumed)
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
    let max_batch = max_batch_craft(&self.player, recipe);
    if max_batch == 0 {
      s.message(t!("action.crafting.empty"))?;
      return Ok(());
    }
    let Some(batch_count) = NumberRequester {
      prompt: t!("action.crafting.how_many"),
      range: Some(1..=max_batch),
      default: Some(1),
      ..
    }
    .request()?
    else {
      return Ok(());
    };
    for _ in 0..batch_count {
      for rs in recipe.steps.iter() {
        if plan_recipe_consumption(&self.player, rs).is_none() {
          s.message(t!("action.crafting.apply_failed", recipe = recipe.name()))?;
          return Ok(());
        }
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
    let Some(consumed) = plan_recipe_consumption(&self.player, recipe) else {
      return false;
    };
    for (item, count) in consumed {
      let (_, mismatch) = self.player.take_items(|candidate| candidate == &item, count);
      if mismatch != 0 {
        return false;
      }
    }
    for ids in recipe.outputs.iter().copied() {
      if let Some(r) = self.player.insert_stack(ids.into()) {
        self.locations[self.player.location].pickup_stacks += r;
      }
    }
    true
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::{
    item::{BERRY, BRANCH},
    recipe::depends::{ItemReq, Req},
  };

  #[test]
  fn max_batch_combines_all_requirements_and_checks_tools() {
    let berry_stack = BERRY.item() * 4;
    let branch_stack = BRANCH.item() * 2;
    let stacks = vec![&berry_stack, &branch_stack];
    const REQUIREMENTS: [Depends; 2] = [
      Depends::Req(Req { item: ItemReq::Def(BERRY), count: 2, consume: ConsumeType::PerRecipe }),
      Depends::Req(Req { item: ItemReq::Def(BRANCH), count: 1, consume: ConsumeType::PerRecipe }),
    ];
    let requirements = Depends::AllOf(&REQUIREMENTS);
    assert_eq!(max_batch(&stacks, &requirements), 2);

    let missing_tool = Req::from(ItemReq::from(BRANCH)).count(3).consume(ConsumeType::None).into();
    assert_eq!(max_batch(&stacks, &missing_tool), 0);
  }

  #[test]
  fn max_batch_craft_accounts_for_intermediate_outputs() {
    const STEPS: [Step; 2] = [
      Step::new(crate::item::BERRY * 1, &[crate::item::BRANCH * 2], 1.s(), 1.0),
      Step::new(crate::item::BRANCH * 3, &[crate::item::LEAF * 1], 1.s(), 1.0),
    ];
    const CRAFT: Crafting = Crafting::new("test", (1, 1.0), &STEPS);
    let inventory = crate::item::ItemStacks(vec![crate::item::BERRY.item() * 4]);

    assert_eq!(max_batch_craft(&inventory, &CRAFT), 2);
  }

  #[test]
  fn max_batch_craft_accounts_for_overlapping_requirements() {
    const STEPS: [Step; 1] = [Step::new(
      Depends::AllOf(&[crate::item::BERRY.into(), crate::item::BERRY.into()]),
      &[crate::item::LEAF * 1],
      1.s(),
      1.0,
    )];
    const CRAFT: Crafting = Crafting::new("test", (1, 1.0), &STEPS);
    let inventory = crate::item::ItemStacks(vec![crate::item::BERRY.item() * 3]);

    assert_eq!(max_batch_craft(&inventory, &CRAFT), 1);
  }

  #[test]
  fn recipe_step_consumes_requirements_and_produces_output() {
    let mut game = Game::default();
    let step = &crate::recipe::crafting::CRAFTING_RECIPES[3].steps[0];
    let pocket = &mut game.player.worn[0].pockets[0];
    pocket.closed = false;
    pocket.stacks += crate::item::LIVING_FISH.item() * 1;
    pocket.recalc_volume_and_size();

    assert!(game.apply_recipe_step(step));
    assert!(game.player.iter_stack().all(|stack| stack.item.def != crate::item::LIVING_FISH));
    assert_eq!(
      game
        .player
        .iter_stack()
        .filter(|stack| stack.item.def == crate::item::RAW_FISH)
        .map(|stack| stack.count)
        .sum::<u64>(),
      1
    );
  }

  #[test]
  fn recipe_step_does_not_consume_partial_requirements() {
    let mut game = Game::default();
    let step = &crate::recipe::crafting::CRAFTING_RECIPES[3].steps[0];
    let pocket = &mut game.player.worn[0].pockets[0];
    pocket.closed = false;
    pocket.stacks += BERRY.item() * 2;
    pocket.recalc_volume_and_size();

    assert!(!game.apply_recipe_step(step));
    assert_eq!(game.player.iter_stack().map(|stack| stack.count).sum::<u64>(), 2);
  }
}

use std::{
  cmp::Reverse,
  rc::Rc,
  sync::atomic::{AtomicU64, Ordering},
};

use itertools::Itertools;

use super::*;
use crate::{
  game::{Game, WaitActionError},
  item::{Item, ItemStack},
  player::{
    Effect, Player,
    action::{Action, ActionContent},
  },
  units::Duration,
  utils::{IterStack, TakeItems},
};

#[cfg(test)]
mod tests;
pub const CRAFTING_RECIPES: &[Crafting] = &[
  Crafting {
    id: "dry_tree_vine",
    batch: (2, 0.01),
    steps: &[Step {
      custom_name: None,
      inputs: Depends::AllOf(&[
        TREE_VINE.into(),
        Req::from(FIRE + 1000).consume(ConsumeType::None).into(),
        (FIRE + 250).into(),
      ]),
      outputs: (&[DRY_TREE_VINE + 1]).into(),
      time: 1.h(),
      activity: 1.3,
    }],
    ..
  },
  Crafting {
    id: "vine_backpack",
    batch: (2, 0.95),
    steps: &[Step {
      inputs: (DRY_TREE_VINE + 5).into(),
      outputs: (&[VINE_BACKPACK + 1]).into(),
      time: 2.h(),
      activity: 1.4,
      ..
    }],
    ..
  },
  Crafting {
    id: "vine_basket",
    batch: (1, 1.0),
    steps: &[Step {
      inputs: (DRY_TREE_VINE + 10).into(),
      outputs: (&[VINE_BASKET + 1]).into(),
      time: 3.h(),
      activity: 1.4,
      ..
    }],
    ..
  },
  Crafting {
    id: "kill_fish",
    batch: (1, 1.0),
    steps: &[Step {
      custom_name: None,
      inputs: (LIVING_FISH + 1).into(),
      outputs: (&[RAW_FISH + 1]).into(),
      time: 20.s(),
      activity: 1.1,
    }],
    ..
  },
  Crafting {
    id: "grill_fish",
    batch: (1, 1.0),
    steps: &[Step {
      custom_name: None,
      inputs: Depends::AllOf(&[
        ItemReq::All(&[RAW_FISH.into(), LIVING_FISH.into()]).into(),
        Req::from(FIRE + 1000).consume(ConsumeType::None).into(),
        (FIRE + 400).into(),
      ]),
      outputs: (&[GRILLED_FISH + 1]).into(),
      time: 20.mnt(),
      activity: 1.2,
    }],
    ..
  },
];

#[derive(Clone)]
pub struct Crafting {
  pub id: &'static str,
  pub custom_name: Option<&'static str> = None,
  pub batch: (usize, f64),
  pub steps: &'static [Step],
}

#[derive(Clone)]
pub struct Step {
  pub custom_name: Option<&'static str> = None,
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

#[derive(Clone)]
pub(crate) struct PausedCraft {
  id: u64,
  recipe: Crafting,
  batch_count: u64,
  step_index: usize,
  batch_index: u64,
  result_index: u64,
  step_initialized: bool,
  per_recipe_reserved: Vec<(ItemStack, bool)>,
  reserved: Vec<(ItemStack, bool)>,
  pending: Option<PendingCraft>,
}

#[derive(Clone)]
struct PendingCraft {
  consumed: Vec<(ItemStack, bool)>,
  output: Vec<ItemStack>,
  progress: Duration,
}

pub(crate) enum CraftStatus {
  Complete,
  Paused(PausedCraft),
  ApplyFailed(PausedCraft),
  Interrupted(String),
}

static NEXT_PAUSED_CRAFT_ID: AtomicU64 = AtomicU64::new(1);

impl PartialEq for PausedCraft {
  fn eq(&self, other: &Self) -> bool { self.id == other.id }
}

impl Depends {
  pub fn max_batch(&self, inventory: &[ItemStack]) -> u64 {
    use Depends::*;
    match self {
    AnyOf(reqs) => reqs.iter().fold(0u64, |batch, dep| batch.max(dep.max_batch(inventory))),
    AllOf(reqs) => reqs.iter().fold(u64::MAX, |batch, dep| batch.min(dep.max_batch(inventory))),
    Req(req) => {
      let matched = inventory
        .iter()
        .filter(|stack| req.item.matches(&stack.item))
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

  fn plan_consumption(
    &self, inventory: &mut Vec<ItemStack>, consumed: &mut Vec<(ItemStack, bool)>, multiplier: u64,
    result_count: u64, result_index: u64, batch_all_pass: bool,
  ) -> bool {
    match self {
    Depends::Req(req) => {
      use ConsumeType::*;
      if matches!(req.consume, BatchAll { .. }) != batch_all_pass {
        return true;
      }
      if !matches!(req.consume, ForEachResult { .. }) && result_index != 0 && req.consume != None {
        return true;
      }
      let consume = match req.consume {
      None => false,
      ForEachResult { .. } => true,
      _ => result_index == 0,
      };
      let needed_count = match req.consume {
      None => req.count,
      ForEachResult { .. } => {
        if result_count == 0 || req.count % result_count != 0 {
          return false;
        }
        (req.count / result_count).saturating_mul(multiplier)
      }
      BatchAll { .. } if result_count > 1 => {
        let base = req.count / result_count;
        let remainder = req.count % result_count;
        (base + u64::from(result_index < remainder)).saturating_mul(multiplier)
      }
      _ => req.count.saturating_mul(multiplier),
      };
      let available = inventory
        .iter()
        .filter(|stack| req.item.matches(&stack.item))
        .map(|stack| stack.count)
        .fold(0u64, |total, count| total.saturating_add(count));
      if available < needed_count {
        return false;
      }
      if !consume {
        return true;
      }
      let mut needed = needed_count;
      for stack in inventory.iter_mut().filter(|stack| req.item.matches(&stack.item)) {
        let take = stack.count.min(needed);
        stack.count -= take;
        needed -= take;
        if take != 0 {
          let return_on_break = req.consume.returns_on_break();
          if let Some((existing, _)) = consumed.iter_mut().find(|(existing, should_return)| {
            existing.item == stack.item && *should_return == return_on_break
          }) {
            existing.count += take;
          } else {
            consumed.push((stack.item.clone() * take, return_on_break));
          }
        }
        if needed == 0 {
          break;
        }
      }
      true
    }
    Depends::AllOf(items) => items.iter().all(|item| {
      item.plan_consumption(
        inventory,
        consumed,
        multiplier,
        result_count,
        result_index,
        batch_all_pass,
      )
    }),
    Depends::AnyOf(items) => {
      let mut alternatives: Vec<_> = items.iter().collect();
      alternatives.sort_by_key(|item| Reverse(item.max_batch(inventory)));
      for item in alternatives {
        let mut candidate_inventory = inventory.clone();
        let mut candidate_consumed = consumed.clone();
        if item.plan_consumption(
          &mut candidate_inventory,
          &mut candidate_consumed,
          multiplier,
          result_count,
          result_index,
          batch_all_pass,
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

  fn valid_result_requirements(&self, result_count: u64) -> bool {
    match self {
    Depends::Req(req) => match req.consume {
    ConsumeType::ForEachResult { .. } => result_count != 0 && req.count % result_count == 0,
    _ => true,
    },
    Depends::AllOf(items) => items.iter().all(|item| item.valid_result_requirements(result_count)),
    Depends::AnyOf(items) => items.iter().any(|item| item.valid_result_requirements(result_count)),
    }
  }
}

impl Crafting {
  pub fn max_batch(&self, items: &impl IterStack) -> u64 {
    let mut inventory: Vec<_> = items.iter_stack().cloned().collect();
    let mut count = u64::MAX;
    for step in self.steps {
      if !step
        .inputs
        .valid_result_requirements(step.outputs.iter().map(|output| output.count).sum())
      {
        return 0;
      }
      let mut upper = count.min(step.inputs.max_batch(&inventory));
      let mut lower = 0;
      while lower < upper {
        let distance = upper - lower;
        let candidate = lower + distance / 2 + distance % 2;
        let mut candidate_inventory = inventory.clone();
        let mut candidate_consumed = Vec::new();
        let can_consume = step.inputs.plan_consumption(
          &mut candidate_inventory,
          &mut candidate_consumed,
          candidate,
          1,
          0,
          false,
        ) && step.inputs.plan_consumption(
          &mut candidate_inventory,
          &mut candidate_consumed,
          candidate,
          1,
          0,
          true,
        );
        if can_consume {
          lower = candidate;
        } else {
          upper = candidate - 1;
        }
      }
      count = lower;
      let mut consumed = Vec::new();
      if !step.inputs.plan_consumption(&mut inventory, &mut consumed, count, 1, 0, false)
        || !step.inputs.plan_consumption(&mut inventory, &mut consumed, count, 1, 0, true)
      {
        return 0;
      }
      for output in step.outputs.iter().copied() {
        let output_count = output.count.saturating_mul(count);
        if let Some(existing) = inventory.iter_mut().find(|stack| stack.item.def == output.item) {
          existing.count = existing.count.saturating_add(output_count);
        } else {
          inventory.push(output.item.item() * output_count);
        }
      }
    }
    count
  }
}

fn is_pause_error(error: &(dyn std::error::Error + 'static)) -> bool {
  matches!(
    error.downcast_ref::<WaitActionError>(),
    Some(WaitActionError::LowEfficiency | WaitActionError::Interrupted)
  )
}

impl Game {
  pub(crate) fn store_paused_craft(&mut self, paused: PausedCraft) {
    let item = Item::from_paused_craft(paused) * 1;
    if let Some(left) = self.player.insert_stack(item) {
      self.locations[self.player.location].pickup_stacks += left;
    }
  }

  pub(crate) fn craft_recipe(
    &mut self, recipe: &Crafting, batch_count: u64,
  ) -> Result<CraftStatus, Box<dyn std::error::Error>> {
    self.continue_craft(PausedCraft {
      id: NEXT_PAUSED_CRAFT_ID.fetch_add(1, Ordering::Relaxed),
      recipe: recipe.clone(),
      batch_count,
      step_index: 0,
      batch_index: 0,
      result_index: 0,
      step_initialized: false,
      per_recipe_reserved: Vec::new(),
      reserved: Vec::new(),
      pending: None,
    })
  }

  pub(crate) fn continue_craft(
    &mut self, mut paused: PausedCraft,
  ) -> Result<CraftStatus, Box<dyn std::error::Error>> {
    while paused.step_index < paused.recipe.steps.len() {
      let step = &paused.recipe.steps[paused.step_index];
      let result_count = step.outputs.iter().map(|output| output.count).sum::<u64>();
      let each_result = step.inputs_has_for_each_result();
      let allocation_result_count = if each_result { result_count } else { 1 };
      if !paused.step_initialized {
        let Some(per_recipe_reserved) = step.plan_batch_all_consumption(&self.player, 1, 1, 0)
        else {
          return Ok(CraftStatus::ApplyFailed(paused));
        };
        let Some(reserved) =
          step.plan_batch_all_consumption(&self.player, paused.batch_count, 1, 0)
        else {
          return Ok(CraftStatus::ApplyFailed(paused));
        };
        if !self.consume_planned_items(&reserved) {
          return Err("crafting inventory changed while reserving batch inputs".into());
        }
        paused.per_recipe_reserved = per_recipe_reserved;
        paused.reserved = reserved;
        paused.step_initialized = true;
      }

      if let Some(mut pending) = paused.pending.take() {
        match self.run_crafting_action(step, pending.progress) {
        Ok(()) => {
          self.produce_outputs(pending.output);
          if each_result {
            paused.subtract_reserved_items(
              paused.batch_all_items_for_result(allocation_result_count, paused.result_index),
            );
          } else {
            paused.subtract_reserved_items(paused.per_recipe_reserved.clone());
          }
          paused.advance(step);
          continue;
        }
        Err(error) if is_pause_error(&*error) => {
          pending.progress = self.take_crafting_progress().unwrap_or(pending.progress);
          paused.pending = Some(pending);
          return Ok(CraftStatus::Paused(paused));
        }
        Err(error) => {
          self.return_items_on_break(pending.consumed);
          self.return_items_on_break(std::mem::take(&mut paused.reserved));
          return Ok(CraftStatus::Interrupted(error.to_string()));
        }
        }
      }

      let Some(consumed) =
        step.plan_recipe_consumption(&self.player, result_count, paused.result_index)
      else {
        self.return_items_on_break(std::mem::take(&mut paused.reserved));
        return Ok(CraftStatus::ApplyFailed(paused));
      };
      if !self.consume_planned_items(&consumed) {
        return Err("crafting inventory changed while consuming recipe inputs".into());
      }
      let output = if each_result {
        let Some(output) = step.result_at(paused.result_index) else {
          return Err("recipe output index is outside its declared results".into());
        };
        vec![output]
      } else {
        step.outputs.iter().map(|output| (*output).into()).collect()
      };
      paused.pending = Some(PendingCraft { consumed, output, progress: Duration::ZERO });
    }
    Ok(CraftStatus::Complete)
  }

  fn run_crafting_action(
    &mut self, step: &Step, progress: Duration,
  ) -> Result<(), Box<dyn std::error::Error>> {
    let activity = step.activity;
    self.player_action(Action {
      id: "crafting",
      f: Rc::new(move |_| ActionContent::new(vec![], vec![Effect::ActivityMul(activity)])),
      progress,
      total_dur: step.time,
    })
  }

  fn take_crafting_progress(&mut self) -> Option<Duration> {
    let index = self.player.actions.iter().position(|action| action.id == "crafting")?;
    Some(self.player.actions.remove(index).progress)
  }

  fn return_items_on_break(&mut self, consumed: Vec<(ItemStack, bool)>) {
    for (stack, _) in consumed.into_iter().filter(|(_, should_return)| *should_return) {
      if let Some(left) = self.player.insert_stack(stack) {
        self.locations[self.player.location].pickup_stacks += left;
      }
    }
  }

  fn produce_outputs(&mut self, outputs: impl IntoIterator<Item = ItemStack>) {
    for output in outputs {
      if let Some(left) = self.player.insert_stack(output) {
        self.locations[self.player.location].pickup_stacks += left;
      }
    }
  }

  fn consume_planned_items(&mut self, consumed: &[(ItemStack, bool)]) -> bool {
    for (stack, _) in consumed {
      let (_, mismatch) = self.player.take_items(|candidate| candidate == &stack.item, stack.count);
      if mismatch != 0 {
        return false;
      }
    }
    true
  }

  pub fn apply_recipe_step(&mut self, recipe: &Step) -> bool {
    let result_count = recipe.outputs.iter().map(|output| output.count).sum::<u64>();
    if !recipe.inputs.valid_result_requirements(result_count) {
      return false;
    }
    if recipe.inputs_has_for_each_result() {
      let mut output_index = 0;
      for output in recipe.outputs.iter() {
        for _ in 0..output.count {
          let Some(consumed) =
            recipe.plan_recipe_consumption(&self.player, result_count, output_index)
          else {
            return false;
          };
          if !self.consume_planned_items(&consumed) {
            return false;
          }
          self.produce_outputs([Item::from(output.item) * 1]);
          output_index += 1;
        }
      }
      return true;
    }
    let Some(consumed) = recipe.plan_recipe_consumption(&self.player, result_count, 0) else {
      return false;
    };
    if !self.consume_planned_items(&consumed) {
      return false;
    }
    self.produce_outputs(recipe.outputs.iter().map(|output| (*output).into()));
    true
  }
}

impl PausedCraft {
  pub(crate) fn recipe_name(&self) -> Cow<'_, str> { self.recipe.name() }

  pub(crate) fn input_volume(&self) -> Volume {
    self
      .reserved
      .iter()
      .chain(self.pending.iter().flat_map(|pending| pending.consumed.iter()))
      .map(|(stack, _)| stack.volume())
      .fold(Volume::ZERO, |total, volume| total + volume)
  }

  pub(crate) fn input_weight(&self) -> crate::units::Mass {
    self
      .reserved
      .iter()
      .chain(self.pending.iter().flat_map(|pending| pending.consumed.iter()))
      .map(|(stack, _)| stack.weight())
      .fold(crate::units::Mass::ZERO, |total, weight| total + weight)
  }

  pub(crate) fn dismantle(self) -> impl Iterator<Item = ItemStack> {
    let reserved =
      self.reserved.into_iter().filter_map(|(stack, _)| (stack.count != 0).then_some(stack));
    let consumed =
      self.pending.into_iter().flat_map(|pending| pending.consumed).map(|(stack, _)| stack);
    reserved.chain(consumed)
  }

  fn advance(&mut self, step: &Step) {
    let result_count = step.outputs.iter().map(|output| output.count).sum::<u64>();
    let output_iterations = if step.inputs_has_for_each_result() { result_count } else { 1 };
    self.result_index += 1;
    if self.result_index >= output_iterations {
      self.result_index = 0;
      self.batch_index += 1;
      if self.batch_index >= self.batch_count {
        self.step_index += 1;
        self.batch_index = 0;
        self.step_initialized = false;
        self.per_recipe_reserved.clear();
        self.reserved.clear();
      }
    }
  }

  fn subtract_reserved_items(&mut self, committed: Vec<(ItemStack, bool)>) {
    for (committed, return_on_break) in committed {
      if let Some((stack, _)) = self.reserved.iter_mut().find(|(stack, should_return)| {
        stack.item == committed.item && *should_return == return_on_break
      }) {
        stack.count = stack.count.saturating_sub(committed.count);
      }
    }
  }

  fn batch_all_items_for_result(
    &self, result_count: u64, result_index: u64,
  ) -> Vec<(ItemStack, bool)> {
    self
      .per_recipe_reserved
      .iter()
      .filter_map(|(stack, should_return)| {
        let base = stack.count / result_count;
        let remainder = stack.count % result_count;
        let count = base + u64::from(result_index < remainder);
        (count != 0).then(|| (stack.item.clone() * count, *should_return))
      })
      .collect()
  }
}

impl Step {
  fn plan_recipe_consumption(
    &self, player: &Player, result_count: u64, result_index: u64,
  ) -> Option<Vec<(ItemStack, bool)>> {
    let mut inventory: Vec<_> = player.iter_stack().cloned().collect();
    let mut consumed = Vec::new();
    self
      .inputs
      .plan_consumption(&mut inventory, &mut consumed, 1, result_count, result_index, false)
      .then_some(consumed)
  }

  fn plan_batch_all_consumption(
    &self, player: &Player, multiplier: u64, result_count: u64, result_index: u64,
  ) -> Option<Vec<(ItemStack, bool)>> {
    let mut inventory: Vec<_> = player.iter_stack().cloned().collect();
    let mut consumed = Vec::new();
    self
      .inputs
      .plan_consumption(&mut inventory, &mut consumed, multiplier, result_count, result_index, true)
      .then_some(consumed)
  }

  fn result_at(&self, index: u64) -> Option<ItemStack> {
    let mut index = index;
    for output in self.outputs.iter() {
      if index < output.count {
        return Some(output.item.item() * 1);
      }
      index -= output.count;
    }
    None
  }

  fn inputs_has_for_each_result(&self) -> bool {
    fn contains(depends: &Depends) -> bool {
      match depends {
      Depends::Req(req) => matches!(req.consume, ConsumeType::ForEachResult { .. }),
      Depends::AllOf(items) | Depends::AnyOf(items) => items.iter().any(contains),
      }
    }
    contains(&self.inputs)
  }
}

impl NameAndDesc for Crafting {
  const PREFIX: &str = "crafting.recipe";

  fn get_id(&self) -> Cow<'_, str> { Cow::Borrowed(self.id) }

  fn name(&self) -> Cow<'_, str> {
    if let Some(n) = self.custom_name {
      t!(n)
    } else if let s = format!("{}.{}.name", Crafting::PREFIX, self.get_id())
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
          if v.is_empty() { None } else { Some(v.into_iter().map(|i| i.name()).join(", ")) }
        })
        .collect_vec();
      if out.is_empty() { t!("crafting.recipe") } else { out.join("; ").into() }
    }
  }

  fn description(&self) -> Cow<'_, str> {
    let mut r = String::new();
    for (index, step) in self.steps.iter().enumerate() {
      let ik = format!("crafting.recipe.{}.step.{}.name", self.id, index);
      let tik = t!(ik);
      _ = r.write_str(&t!(
        "crafting.recipe.step",
        index = index,
        name = if tik != ik { tik } else { step.outputs.iter().map(|o| o.item.name()).join(", ").into() },
        inputs = step.inputs,
        outputs = step.outputs.iter().map(|i| i.item.name()).join(", "),
        time = step.time,
        activity = step.activity: {:.2}
      ));
    }
    r.into()
  }
}

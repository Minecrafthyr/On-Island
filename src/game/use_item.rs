use std::rc::Rc;

use itertools::Itertools;

use crate::{
  game::Game,
  io::ScreenWriter,
  player::{
    Effect,
    action::{Action, ActionContent},
  },
  recipe::crafting::CraftStatus,
  ui::{DataItem, DisplayList, NumberRequester},
  utils::{IterStack, NameAndDesc, TakeItems},
};

impl Game {
  pub fn use_item(&mut self) -> Result<(), Box<dyn std::error::Error>> {
    let options: Vec<_> = self
      .player
      .iter_stack()
      .filter(|stack| !stack.item.uses.is_empty() || stack.item.paused_craft.is_some())
      .cloned()
      .collect();
    if options.is_empty() {
      return Ok(());
    }
    let mut s = ScreenWriter::new_screen();
    let Some(choice) = s.list(DisplayList::new(
      Some(t!("action.use_item.title")),
      options
        .iter()
        .map(|is| DataItem {
          text: t!(
            "action.use_item.entry",
            name = is.item.name(),
            count = is.count,
            uses = is.item.uses.iter().map(|ud| ud.name()).join(" "),
            description = is.item.description()
          ),
          selected: (),
          enter: |i| Ok(i),
        })
        .collect(),
    ))?
    else {
      return Ok(());
    };
    let item_stack = options[choice].clone();
    let item = &item_stack.item;
    if item.paused_craft.is_some() {
      let actions = [t!("action.use_item.paused.continue"), t!("action.use_item.paused.dismantle")];
      let Some(action) = s.list(DisplayList {
        title: Some(t!("action.use_item.paused.title", item = item.name())),
        data: actions
          .into_iter()
          .enumerate()
          .map(|(index, text)| DataItem { text, selected: (), enter: move |_| Ok(index == 0) })
          .collect(),
        indexed: true,
        ..
      })?
      else {
        return Ok(());
      };

      let (mut removed, mismatch) = self.player.take_items(|candidate| candidate == item, 1);
      if mismatch != 0 {
        return Err("the selected paused craft is no longer in the inventory".into());
      }
      let mut stack = removed.pop().ok_or("the selected paused craft could not be removed")?;
      if action {
        let paused = stack
          .item
          .paused_craft
          .as_deref()
          .ok_or("the selected item is not a paused craft")?
          .clone();
        match self.continue_craft(paused)? {
        CraftStatus::Complete => {}
        CraftStatus::Paused(paused) => {
          self.store_paused_craft(paused);
        }
        CraftStatus::ApplyFailed(paused) => {
          let recipe_name = paused.recipe_name().into_owned();
          self.store_paused_craft(paused);
          s.message(t!("action.crafting.apply_failed", recipe = recipe_name))?;
        }
        CraftStatus::Interrupted(message) => {
          s.message(message)?;
        }
        }
      } else {
        let paused =
          stack.item.paused_craft.take().ok_or("the selected item is not a paused craft")?;
        for returned in paused.dismantle() {
          if let Some(left) = self.player.insert_stack(returned) {
            self.locations[self.player.location].pickup_stacks += left;
          }
        }
        s.message(t!("action.use_item.paused.dismantled", item = item.name()))?;
      }
      return Ok(());
    }
    let uses_data = item.uses;
    let Some(choice) = s.list(DisplayList {
      title: Some(t!("action.use_item.data.title", item = item.name())),
      data: uses_data
        .iter()
        .map(|ud| DataItem {
          text: t!(
            "action.use_item.data",
            duration = ud.dur.as_s_f64() : {:.2},
            activity = ud.activity: {:.2},
            description = ud.description()
          ),
          selected: (),
          enter: |i| Ok(i),
        })
        .collect(),
      indexed: true,
      ..
    })?
    else {
      return Ok(());
    };
    let use_data = uses_data[choice];

    let Some(count) = NumberRequester {
      prompt: t!("action.use_item.how_many"),
      range: Some(1..=item_stack.count),
      default: Some(1),
      ..
    }
    .request()?
    else {
      return Ok(());
    };
    for _ in 0..count {
      match self.player_action(Action {
        id: "use_item",
        f: Rc::new(move |_| {
          ActionContent::new(vec![], vec![Effect::ActivityMul(use_data.activity)])
        }),
        total_dur: use_data.dur,
        ..
      }) {
      Ok(()) => {
        let (_removed, mismatch) = self.player.take_items(|i| i == item, 1);
        if mismatch == 0 {
          (use_data.on_use)(&mut self.player);
          return Ok(());
        }
      }
      Err(not_ok) => {
        s.message(not_ok.to_string())?;
        return Ok(());
      }
      }
    }
    Ok(())
  }
}

use std::sync::Arc;

use itertools::Itertools;

use crate::{
  game::Game,
  io::ScreenWriter,
  player::{
    Effect,
    action::{Action, ActionContent},
  },
  ui::{DataItem, DisplayList, NumberRequester},
  utils::{NameAndDesc, TakeItems},
};

impl Game {
  pub fn use_item(&mut self) -> Result<(), Box<dyn std::error::Error>> {
    let options: Vec<_> = self
      .player
      .worn
      .iter_mut()
      .flat_map(|c| c.pockets.iter())
      .flat_map(|p| p.stacks.iter())
      .filter(|is| !is.item.uses.is_empty())
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
        f: Arc::new(move |_| {
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

use itertools::Itertools;

use crate::{
  game::Game,
  io::ScreenWriter,
  player::{
    Effect,
    action::{Action, ActionContent},
  },
  ui::{DataItem, DisplayList, NumberRequester},
  utils::NameAndDesc,
};

impl Game {
  pub fn use_item(&mut self) {
    let options: Vec<_> = self
      .player
      .worn
      .iter_mut()
      .flat_map(|c| c.pockets.iter())
      .flat_map(|p| p.stacks.iter())
      .filter(|is| !is.item.uses.is_empty())
      .collect();
    if options.is_empty() {
      return;
    }
    let mut s = ScreenWriter::new_screen();
    let Some(choice) = s.list(DisplayList::new(
      Some(t!("action.use_item.title")),
      options
        .iter()
        .enumerate()
        .map(|(i, is)| DataItem {
          text: t!(
            "action.use_item.entry",
            index = i,
            name = is.item.name(),
            count = is.count,
            uses = is.item.uses.iter().map(|ud| ud.name()).join(" "),
            description = is.item.description()
          ),
          selected: (),
          enter: |i| Ok(i),
        })
        .collect(),
    )) else {
      return;
    };
    let item_stack = options[choice].clone();
    let item = &item_stack.item;
    let uses_data = item.uses;
    let Some(choice) = s.list(DisplayList::new(
      Some(t!("action.use_item.data.title", item = item.name())),
      uses_data
        .iter()
        .enumerate()
        .map(|(i, ud)| DataItem {
          text: t!(
            "action.use_item.data",
            index = i,
            duration = ud.dur.as_seconds_f64() : {:.2},
            activity = ud.activity: {:.2},
            description = ud.description()
          ),
          selected: (),
          enter: |i| Ok(i),
        })
        .collect(),
    )) else {
      return;
    };
    let use_data = uses_data[choice];

    let Some(count) = NumberRequester::new(t!("action.use_item.how_many"))
      .range(1..=item_stack.count)
      .default(1)
      .request()
    else {
      return;
    };
    for _ in 0..count {
      match self.player_action(Action::no_progress(
        "use_item",
        move |_| ActionContent::new(vec![], vec![Effect::ActivityMul(use_data.activity)]),
        use_data.dur,
      )) {
      Ok(()) => {
        let (_removed, mismatch) = self.player.take_items_matching(|i| i == item, 1);
        if mismatch == 0 {
          (use_data.on_use)(&mut self.player);
          return;
        }
      }
      Err(not_ok) => {
        s.message(not_ok.to_string());
        return;
      }
      }
    }
  }
}

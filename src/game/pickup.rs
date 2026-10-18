use std::sync::Arc;

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
  pub fn pickup(&mut self) {
    let loc = self.player.location;
    let mut s = ScreenWriter::new_screen();
    if self.locations[loc].pickup_stacks.is_empty() {
      s.message(t!("action.pickup.none"));
      return;
    }
    let options: Vec<_> = self.locations[loc]
      .pickup_stacks
      .iter()
      .filter_map(|item_stack| {
        let item = item_stack.item.clone();
        self
          .player
          .pickup_time(&item)
          .map(|(dur, activity)| (item, item_stack.count, dur, activity))
      })
      .collect();

    let Some(choice) = s.list(
      DisplayList::new(
        Some(t!("action.pickup.title")),
        options
          .iter()
          .enumerate()
          .map(|(_, (item, count, dur, activity))| DataItem {
            text: t!(
              "action.pickup.entry",
              name = item.name(),
              count = count,
              duration = dur.as_s_f64() : {:.2},
              activity = activity : {:.2},
              description = item.description()
            ),
            selected: (),
            enter: |i| Ok(i),
          })
          .collect(),
      )
      .indexing(),
    ) else {
      return;
    };

    let (item, stack_count, _, _) = options[choice].clone();
    let Some(count) = NumberRequester::new(t!("action.pickup.how_many"))
      .range(1..=stack_count)
      .default(1)
      .request()
    else {
      return;
    };
    for i in 0..count {
      let Some((pick_time, activity)) = self.player.pickup_time(&item) else {
        s.message(t!("action.pickup.remaining", count = count - i));
        return;
      };
      match self.player_action(Action {
        id: "gather",
        f: Arc::new(move |_| ActionContent::new(vec![], vec![Effect::ActivityMul(activity)])),
        total_dur: pick_time,
        ..
      }) {
      Ok(()) => {
        if let Some(_left) = self.player.insert_stack(item.clone() * 1) {
          s.message(t!("action.pickup.remaining", count = count - i));
          return;
        };
        self.locations[loc].pickup_stacks.take_items(&item, 1);
      }
      Err(not_ok) => {
        s.message(not_ok.to_string());
        return;
      }
      }
    }
  }
}

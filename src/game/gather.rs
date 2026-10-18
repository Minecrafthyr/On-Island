use std::sync::Arc;

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
  pub fn gather(&mut self) {
    let loc = &self.locations[self.player.location];
    let mut s = ScreenWriter::new_screen();
    let Some(choice) = s.list(
      DisplayList::new(
        Some(t!("action.gather.title")),
        loc
          .gather_stacks
          .iter()
          .map(|item_stack| DataItem {
            text: t!(
              "action.gather.entry",
              name = item_stack.item.name(),
              count = loc.gather_stacks[&item_stack.item].count,
              description = item_stack.item.description(),
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
    let item_stack = loc.gather_stacks[choice].clone();

    let Some(count) = NumberRequester::new(t!("action.gather.how_many"))
      .range(1..=item_stack.count)
      .default(1)
      .request()
    else {
      return;
    };

    loop {
      use crate::item::Gather::*;
      match item_stack.item.gather {
      None => return,
      Single { time, activity, f } =>
        for _ in 0..count {
          match self.player_action(Action {
            id: "gather",
            f: Arc::new(move |_| ActionContent::new(vec![], vec![Effect::ActivityMul(activity)])),
            total_dur: time,
            ..
          }) {
          Ok(()) => {
            let loc = &mut self.locations[self.player.location];
            self.player.insert_stacks(f());
            loc.gather_stacks[choice].count -= 1;
          }
          Err(not_ok) => {
            s.message(not_ok.to_string());
            return;
          }
          }
        },
      Multiple { time, activity, f } => {
        match self.player_action(Action {
          id: "gather",
          f: Arc::new(move |_| ActionContent::new(vec![], vec![Effect::ActivityMul(activity)])),
          total_dur: time,
          ..
        }) {
        Ok(()) => {
          let loc = &mut self.locations[self.player.location];
          self.player.insert_stacks(f(count));
          loc.gather_stacks[choice].count -= 1;
        }
        Err(not_ok) => {
          s.message(not_ok.to_string());
          return;
        }
        }
      }
      }
    }
  }
}

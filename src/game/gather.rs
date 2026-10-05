use rand::RngExt;

use crate::{
  game::{Duration, Game, ToDuration},
  io::ScreenWriter,
  location::ResData,
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
    let Some(choice) = s.list(DisplayList::new(
      Some(t!("action.gather.title")),
      loc
        .res
        .iter()
        .enumerate()
        .map(|(i, item_stack)| DataItem {
          text: t!(
            "action.gather.entry",
            index = i,
            name = item_stack.item.name(),
            count = loc.gather_stacks[&item_stack.item].count,
            description = item_stack.item.description(),
          ),
          selected: (),
          enter: |i| Ok(i),
        })
        .collect(),
    )) else {
      return;
    };
    let ResData { item, gather_time, activity, .. } = loc.res[choice].clone();

    let Some(count) = NumberRequester::new(t!("action.gather.how_many"))
      .range(1..=loc.gather_stacks[&loc.res[choice].item].count)
      .default(1)
      .request()
    else {
      return;
    };

    loop {
      use crate::item::GatherStacks::*;
      match item.gather {
      None => return,
      Single(f) =>
        for _ in 0..count {
          let (b, e) = gather_time.clone().into_inner();
          let t = self.rng.random_range(b.as_us()..=e.as_us());
          match self.player_action(Action::no_progress(
            "gather",
            move |_| ActionContent::new(vec![], vec![Effect::ActivityMul(activity)]),
            t.us(),
          )) {
          Ok(()) => {
            let loc = &mut self.locations[self.player.location];
            self.player.insert_stacks(f(&mut loc.res[choice]));
            loc.gather_stacks[&loc.res[choice].item].count -= 1;
          }
          Err(not_ok) => {
            s.message(not_ok.to_string());
            return;
          }
          }
        },
      Multiple(f) => {
        let (b, e) = gather_time.clone().into_inner();
        let t = self.rng.random_range(b.as_us()..=e.as_us());
        match self.player_action(Action::no_progress(
          "gather",
          move |_| ActionContent::new(vec![], vec![Effect::ActivityMul(activity)]),
          Duration::us(t),
        )) {
        Ok(()) => {
          let loc = &mut self.locations[self.player.location];
          self.player.insert_stacks(f(&mut loc.res[choice], count));
          loc.gather_stacks[&loc.res[choice].item].count -= 1;
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

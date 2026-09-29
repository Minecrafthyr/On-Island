use rand::RngExt;

use crate::{
  game::{Duration, Game},
  io::ScreenWriter,
  location::RestorationData,
  player::{
    Effect,
    action::{Action, ActionContent},
  },
  ui::{DataItem, DisplayList, NumberRequester},
  utils::NameAndDesc,
};
impl Game {
  pub fn gather(&mut self) {
    let location_data = &self.locations[self.player.location];
    let mut s = ScreenWriter::new_screen();
    let Some(choice) = s.list(DisplayList::new(
      Some(t!("action.gather.title")),
      location_data
        .restore_and_gather
        .iter()
        .enumerate()
        .map(|(i, item_stack)| DataItem {
          text: t!(
            "action.gather.entry",
            index = i,
            name = item_stack.item.name(),
            count = item_stack.count,
            description = item_stack.item.description(),
          ),
          selected: (),
          enter: |i| Ok(i),
        })
        .collect(),
    )) else {
      return;
    };
    let RestorationData { item, gather_time, activity, .. } =
      location_data.restore_and_gather[choice].clone();

    let Some(count) = NumberRequester::new(t!("action.gather.how_many"))
      .range(1..=location_data.restore_and_gather[choice].count)
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
          let t = self.rng.random_range(b.as_s_f64()..=e.as_s_f64());
          match self.player_action(Action::no_progress(
            "gather",
            move |_| ActionContent::new(vec![], vec![Effect::ActivityMul(activity)]),
            Duration::from_s_f64(t),
          )) {
          Ok(()) => {
            self.player.insert_stacks(f(
              &mut self.locations[self.player.location].restore_and_gather[choice]
            ));
            self.locations[self.player.location].restore_and_gather[choice].count -= 1;
          }
          Err(not_ok) => {
            s.message(not_ok.to_string());
            return;
          }
          }
        },
      Multiple(f) => {
        let (b, e) = gather_time.clone().into_inner();
        let t = self.rng.random_range(b.as_s_f64()..=e.as_s_f64());
        match self.player_action(Action::no_progress(
          "gather",
          move |_| ActionContent::new(vec![], vec![Effect::ActivityMul(activity)]),
          Duration::from_s_f64(t),
        )) {
        Ok(()) => {
          self.player.insert_stacks(f(
            &mut self.locations[self.player.location].restore_and_gather[choice],
            count,
          ));
          self.locations[self.player.location].restore_and_gather[choice].count -= 1;
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

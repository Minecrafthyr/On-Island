use rand::RngExt;
use time::Duration;

use crate::{
  game::Game,
  io::ScreenWriter,
  location::RestorationData,
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
    let RestorationData { item: _, gather_time, activity, produce, .. } =
      location_data.restore_and_gather[choice].clone();

    let Some(count) = NumberRequester::new(t!("action.gather.how_many"))
      .range(1..=location_data.restore_and_gather[choice].count)
      .default(1)
      .request()
    else {
      return;
    };
    for _ in 0..count {
      let (b, e) = gather_time.clone().into_inner();
      let t = self.rng.random_range(b.as_seconds_f64()..=e.as_seconds_f64());
      self.action_time_pass(Duration::seconds_f64(t), activity);
      self.player.insert_stacks(produce(
        &mut self.locations[self.player.location].restore_and_gather[choice],
      ));

      self.locations[self.player.location].restore_and_gather[choice].count -= 1;
    }
  }
}

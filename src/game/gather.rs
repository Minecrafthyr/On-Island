use rand::RngExt;
use time::Duration;

use crate::{
  game::Game,
  io::NewScreenWriter,
  item::{Item, ItemStacks, RAW_FISH, TREE},
  ui::{DataItem, DisplayList, NumberRequester},
  utils::NameAndDesc,
};
impl Game {
  pub fn gather(&mut self) {
    let location_data = &self.locations[self.player.location];
    let mut s = NewScreenWriter::new();
    let Some(choice) = s.list(DisplayList::new(
      Some(t!("action.gather.title")),
      location_data
        .gather_stacks
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
    let item = location_data.gather_stacks[choice].item.clone();
    let Some((gather_time, activity, item_stacks)) = (match item.get_id() {
    "raw_fish" => Some((
      Duration::seconds(self.rng.random_range(120..=2000)),
      1.2,
      ItemStacks::from([(Item::new(RAW_FISH), 1)]),
    )),
    "tree" => Some((Duration::minutes(30), 2.0, [(Item::new(TREE), 2)].into())),
    _ => None,
    }) else {
      s.message("你无法采集它！");
      return;
    };
    let Some(count) = NumberRequester::new(t!("action.gather.how_many"))
      .range(1..=location_data.gather_stacks[choice].count)
      .default(1)
      .request()
    else {
      return;
    };
    for _ in 0..count {
      self.action_time_pass(gather_time, activity);
      for item_stack in item_stacks.iter().cloned() {
        self.player.inventory.insert_items(item_stack);
      }
      self.locations[self.player.location].gather_stacks.remove_items(&item, 1);
    }
  }
}

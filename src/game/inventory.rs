
use crate::{
  game::Game,
  io::NewScreenWriter,
  ui::{DisplayList, popup_message},
  utils::NameAndDesc,
};
impl Game {
  pub fn inventory(&self) {
    NewScreenWriter::new();
    if self.player.inventory.is_empty() {
      popup_message("物品栏里什么也没有！");
      return;
    }
    DisplayList::new(
      Some(t!("action.inventory.title")),
      self
        .player
        .inventory
        .iter()
        .map(|is| {
          (
            t!(
              "action.inventory.unselected",
              name = is.item.name(),
              count = is.count,
              volume = is.volume(),
              weight = is.weight()
            ),
            Some(t!(
              "action.inventory.selected",
              name = is.item.name(),
              count = is.count,
              volume = is.volume(),
              weight = is.weight(),
              description = is.item.description()
            )),
            None,
          )
        })
        .collect(),
    )
    .run();
  }
}

use crate::{
  game::Game,
  io::NewScreenWriter,
  ui::{DataItem, DisplayList},
  utils::NameAndDesc,
};
impl Game {
  pub fn inventory(&self) {
    let mut s = NewScreenWriter::new();
    s.list(DisplayList::new(
      Some(t!("action.inventory.title")),
      self
        .player
        .inventory
        .iter()
        .map(|is| DataItem {
          text: t!(
            "action.inventory.entry",
            name = is.item.name(),
            count = is.count,
            volume = is.volume(),
            weight = is.weight(),
            description = is.item.description()
          ),
          selected: (),
          enter: (),
        })
        .collect(),
    ));
  }
}

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
        .worn
        .iter()
        .filter_map(|i| i.container.as_ref())
        .flat_map(|c| c.pockets.iter().flat_map(|p| p.stacks.iter()))
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

use crate::{
  game::Game,
  io::ScreenWriter,
  ui::{DataItem, DisplayList},
  utils::NameAndDesc,
};
impl Game {
  pub fn inventory(&self) {
    let mut s = ScreenWriter::new_screen();
    s.list(DisplayList::new(
      Some(t!("action.inventory.title")),
      self
        .player
        .worn
        .iter()
        .flat_map(|i| i.pockets.iter().flat_map(|p| p.stacks.iter()))
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

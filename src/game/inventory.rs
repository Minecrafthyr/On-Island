use crate::{
  game::Game,
  io::ScreenWriter,
  ui::{DataItem, DisplayList},
  utils::NameAndDesc,
};
impl Game {
  // TODO: implement interactions
  pub fn inventory(&self) {
    ScreenWriter::new_screen().list(
      DisplayList::new(
        Some(t!("action.inventory.title")),
        self
          .player
          .worn
          .iter()
          .flat_map(|i| &i.pockets)
          .flat_map(|p| p.stacks.iter())
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
      )
      .indexing(),
    );
  }
}

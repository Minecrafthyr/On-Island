use crossterm::style::Stylize;

use crate::{
  game::Game,
  io::ScreenWriter,
  preclude::*,
  recipe::crafting::{CRAFTING_RECIPES, CraftStatus},
  ui::{DataItem, DisplayList, NumberRequester},
};

impl Game {
  pub fn craft(&mut self) -> Result<(), Box<dyn std::error::Error>> {
    let options: Vec<_> = CRAFTING_RECIPES.iter().collect();
    let mut screen = ScreenWriter::new_screen();
    let Some(choice) = screen.list(DisplayList {
      title: Some(t!("action.crafting.title")),
      data: options
        .iter()
        .enumerate()
        .map(|(index, recipe)| {
          let name = recipe.name();
          let available = recipe.max_batch(&self.player) != 0;
          DataItem {
            text: t!(
              "action.craft.entry",
              index = index,
              name = if available { name.stylize() } else { name.grey() }
            ),
            selected: |option: &Cow<'_, str>| {
              Cow::Owned(option.to_string() + "\n" + &recipe.description())
            },
            enter: |index| Ok(index),
          }
        })
        .collect(),
      ..
    })?
    else {
      return Ok(());
    };

    let recipe = options[choice];
    let max_batch = recipe.max_batch(&self.player);
    if max_batch == 0 {
      screen.message(t!("action.crafting.empty"))?;
      return Ok(());
    };
    let Some(batch_count) = NumberRequester {
      prompt: t!("action.crafting.how_many"),
      range: Some(1..=max_batch),
      default: Some(1),
      ..
    }
    .request()?
    else {
      return Ok(());
    };

    match self.craft_recipe(recipe, batch_count)? {
    CraftStatus::Complete => {}
    CraftStatus::Paused(paused) => {
      self.store_paused_craft(paused);
      _ = screen.message(t!("action.crafting.paused"))?;
    }
    CraftStatus::ApplyFailed(paused) => {
      self.store_paused_craft(paused);
      screen.message(t!("action.crafting.apply_failed", recipe = recipe.name()))?;
    }
    CraftStatus::Interrupted(message) => _ = screen.message(message)?,
    }
    Ok(())
  }
}

use crate::{
  crafting::{CRAFTING_RECIPES, Recipe},
  game::Game,
  io::ScreenWriter,
  item::{Item, ItemStack},
  player::Player,
  ui::{DataItem, DisplayList, NumberRequester},
};
impl Game {
  pub fn craft(&mut self) {
    let options: Vec<_> = CRAFTING_RECIPES
      .iter()
      .filter(|recipe| {
        recipe.inputs.iter().all(|ids| self.player.count_of(&Item::new(ids.item)) >= ids.count)
      })
      .collect();
    let mut s = ScreenWriter::new_screen();
    let Some(choice) = s.list(DisplayList::new(
      Some(t!("action.craft.title")),
      options
        .iter()
        .enumerate()
        .map(|(i, recipe)| DataItem {
          text: t!(
            "action.craft.entry",
            index = i,
            title = recipe.name(),
            inputs = recipe.inputs_text(),
            outputs = recipe.outputs_text(),
            duration = recipe.time.as_seconds_f64() : {:.2},
            activity = recipe.activity: {:.2}
          ),
          selected: (),
          enter: |i| Ok(i),
        })
        .collect(),
    )) else {
      return;
    };
    let recipe = options[choice];
    pub fn max_batch_count(r: &Recipe, player: &Player) -> u64 {
      r.inputs
        .iter()
        .map(|ids| {
          let available =
            player.worn.iter().filter_map(|i| i.container.as_ref()).fold(0u64, |i, c| {
              i + c.pockets.iter().fold(0u64, |i, p| i + p.stacks.count_of(&Item::new(ids.item)))
            });
          available / ids.count
        })
        .min()
        .unwrap_or(0)
    }
    let Some(batch_count) = NumberRequester::new(t!("action.craft.how_many"))
      .range(1..=max_batch_count(recipe, &self.player))
      .default(1)
      .request()
    else {
      return;
    };
    for _ in 0..batch_count {
      self.action_time_pass(recipe.time, recipe.activity);
      self.apply_recipe(recipe);
    }
  }

  pub fn apply_recipe(&mut self, recipe: &Recipe) -> bool {
    for ids in recipe.inputs.iter().copied() {
      self.player.insert_items(ItemStack::new(ids.item.into(), ids.count));
    }
    for ids in recipe.outputs.iter().copied() {
      if let Some(r) = self.player.insert_items(ids.into()) {
        self.locations[self.player.location].pickup_stacks.insert_items(r);
      }
    }
    true
  }
}

use crate::{
  crafting::{CRAFTING_RECIPES, CraftingRecipe},
  game::Game,
  io::NewScreenWriter,
  ui::{DataItem, DisplayList, NumberRequester},
};
impl Game {
  pub fn craft(&mut self) {
    let options: Vec<_> =
      CRAFTING_RECIPES.iter().filter(|recipe| recipe.can_apply(&self.player.inventory)).collect();
    let mut s = NewScreenWriter::new();
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
    let Some(batch_count) = NumberRequester::new(t!("action.craft.how_many"))
      .range(1..=recipe.max_batch_count(&self.player.inventory))
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

  pub fn apply_recipe(&mut self, recipe: &CraftingRecipe) -> bool {
    for ids in recipe.inputs.iter().copied() {
      self.player.inventory.remove_items(ids.into());
    }
    for ids in recipe.outputs.iter().copied() {
      if let Some(r) = self.player.insert_items(ids.into()) {
        self.locations[self.player.location].pickup_stacks.insert_items(r);
      }
    }
    true
  }
}

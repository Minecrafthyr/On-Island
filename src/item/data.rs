use std::sync::LazyLock;

use rand::random_range;

use super::*;
use crate::location::EnvCondition;

pub const SAND: &ItemDef = &ItemDef { id: "sand", volume: (65.nL()), weight: (170.ug()), .. };
pub const BISCUIT: &ItemDef = &ItemDef {
  id: "biscuit",
  volume: (100.mL()),
  weight: (20.g()),
  uses: (&[UseData {
    usage: "eat",
    dur: 5.s(),
    batch: (3, 0.7),
    activity: 1.03,
    on_use: |player| {
      player.energy += 1.h();
      player.water -= 1.h();
    },
  }]),
  ..
};
pub const BERRY: &ItemDef = &ItemDef {
  id: "berry",
  volume: (6.mL()),
  weight: (3.g()),
  uses: (&[UseData {
    usage: "eat",
    dur: 100.ms(),
    batch: (2, 0.5),
    activity: 1.02,
    on_use: |player| {
      player.energy += 2.mnt();
      player.water += 1.mnt();
    },
  }]),
  ..
};
pub const BERRY_BRANCH: &ItemDef =
  &ItemDef { id: "berry_branch", volume: (300.mL()), weight: (200.g()), .. };
pub const BERRY_BUSH: &ItemDef =
  &ItemDef { id: "berry_bush", volume: (6.L()), weight: (2.kg()), .. };
pub const BERRY_BUSH_HARVESTED: &ItemDef =
  &ItemDef { id: "berry_bush_harvested", volume: (5.L()), weight: (1200.g()), .. };
pub const WATER: &ItemDef = &ItemDef {
  id: "water",
  volume: (1.uL()),
  weight: (1.mg()),
  phase: (Phase::Liquid),
  uses: (&[UseData {
    usage: "drink",
    dur: 10.ms(),
    batch: (2, 0.0),
    activity: 1.01,
    on_use: |player| {
      player.water += 30.s();
    },
  }]),
  ..
};
pub const SEA_WATER: &ItemDef = &ItemDef {
  id: "sea_water",
  volume: (1.uL()),
  weight: (1025.ug()),
  phase: (Phase::Liquid),
  uses: (&[UseData {
    usage: "drink",
    dur: 10.ms(),
    batch: (2, 0.0),
    activity: 1.01,
    on_use: |player| {
      player.water -= 20.s();
    },
  }]),
  ..
};
pub const PLASTIC_BOTTLE: &ItemDef = &ItemDef {
  id: "plastic_bottle",
  volume: (521.mL()),
  weight: (10.g()),
  pockets: (&[PocketDef {
    id: "plastic_bottle.main",
    capacity: 500.mL(),
    max_weight: 900.g(),
    rigid: true,
    can_store_liquid: true,
    ..
  }]),
  ..
};
pub const LIVING_FISH: &ItemDef = &ItemDef {
  id: "living_fish",
  volume: (1.L()),
  weight: (1200.g()),
  gather: (Gather::Single {
    time: 120.s(),
    activity: 1.2,
    f: || ItemStacks(vec![RAW_FISH.item() * 1]),
  }),
  uses: (&[UseData {
    usage: "eat",
    dur: 135.s(),
    batch: (1, 1.0),
    activity: 1.1,
    on_use: |player| {
      player.energy += 2.h();
      player.water += 55.mnt();
    },
  }]),
  conversions: (&[ConversionDef::new(EnvCondition::AlwaysTrue, 10.mnt(), || {
    ItemStacks(vec![RAW_FISH.item() * 1])
  })]),
  ..
};
pub const RAW_FISH: &ItemDef = &ItemDef {
  id: "raw_fish",
  volume: (1.L()),
  weight: (1200.g()),
  uses: (&[UseData {
    usage: "eat",
    dur: 115.s(),
    batch: (1, 1.0),
    activity: 1.06,
    on_use: |player| {
      player.energy += 2.h();
      player.water += 50.mnt();
    },
  }]),
  ..
};
pub const GRILLED_FISH: &ItemDef = &ItemDef {
  id: "grilled_fish",
  volume: (900.mL()),
  weight: (1050.g()),
  uses: (&[UseData {
    usage: "eat",
    dur: 105.s(),
    batch: (1, 1.0),
    activity: 1.06,
    on_use: |player| {
      player.energy += 4.h();
      player.water += 20.mnt();
    },
  }]),
  ..
};
pub const TREE: &ItemDef = &ItemDef {
  id: "tree",
  volume: (150.L()),
  weight: (100.kg()),
  gather: (Gather::Single {
    time: 120.s(),
    activity: 1.2,
    f: || {
      ItemStacks(vec![
        WOOD.item() * 2,
        BIG_BRANCH.item() * random_range(1..2),
        BRANCH.item() * random_range(30..60),
        STICK.item() * random_range(0..4),
        LEAF.item() * random_range(10..200),
      ])
    },
  }),
  ..
};
pub const WOOD: &ItemDef = &ItemDef { id: "wood", volume: (50.L()), weight: (20.kg()), .. };
pub const SMALL_BRANCH: &ItemDef =
  &ItemDef { id: "small_branch", volume: (600.mL()), weight: (400.g()), .. };
pub const SHORT_STICK: &ItemDef =
  &ItemDef { id: "short_stick", volume: (500.mL()), weight: (250.g()), .. };
pub const BRANCH: &ItemDef = &ItemDef { id: "branch", volume: (1300.mL()), weight: (1.kg()), .. };
pub const STICK: &ItemDef = &ItemDef { id: "stick", volume: (1.L()), weight: (800.g()), .. };
pub const BIG_BRANCH: &ItemDef =
  &ItemDef { id: "big_branch", volume: (2500.mL()), weight: (1500.g()), .. };
pub const LONG_STICK: &ItemDef =
  &ItemDef { id: "long_stick", volume: (2.L()), weight: (1.kg()), .. };
pub const LEAF: &ItemDef = &ItemDef { id: "leaf", volume: (40.mL()), weight: (10.g()), .. };
pub const TREE_VINE: &ItemDef = &ItemDef {
  id: "tree_vine",
  volume: (500.mL()),
  weight: (300.g()),
  gather: (Gather::Single {
    time: 15.s(),
    activity: 1.1,
    f: || ItemStacks(vec![TREE_VINE.item() * 1]),
  }),
  ..
};
pub const DRY_TREE_VINE: &ItemDef =
  &ItemDef { id: "dry_tree_vine", volume: (400.mL()), weight: (200.g()), .. };
pub const FLINT: &ItemDef = &ItemDef { id: "flint", volume: (700.mL()), weight: (800.g()), .. };
pub const ROCK: &ItemDef = &ItemDef { id: "rock", volume: (800.mL()), weight: (1.kg()), .. };
pub const BIG_ROCK: &ItemDef =
  &ItemDef { id: "big_rock", volume: (150.L()), weight: (100.kg()), .. };
pub const VINE_BACKPACK: &ItemDef = &ItemDef {
  id: "vine_backpack",
  volume: (25.L()),
  weight: (900.g()),
  wearable: true,
  pockets: (&[PocketDef {
    id: "vine_backpack.main",
    capacity: 18.L(),
    max_weight: 10.kg(),
    rigid: false,
    ..
  }]),
  ..
};
pub const VINE_BASKET: &ItemDef = &ItemDef {
  id: "vine_basket",
  volume: (L(50)),
  weight: (g(1500)),
  pockets: (&[PocketDef {
    id: "vine_basket.main",
    capacity: 48.L(),
    max_weight: 40.kg(),
    rigid: true,
    ..
  }]),
  ..
};
pub const CANVAS_BACKPACK: &ItemDef = &ItemDef {
  id: "canvas_backpack",
  volume: (L(28)),
  weight: (g(700)),
  wearable: true,
  pockets: (&[PocketDef {
    id: "canvas_backpack.main",
    capacity: 20.L(),
    max_weight: 50.kg(),
    rigid: true,
    ..
  }]),
  ..
};
pub const COTTON_UNDERWEAR: &ItemDef = &ItemDef {
  id: "cotton_underwear",
  volume: (5.L()),
  weight: (1200.g()),
  wearable: true,
  pockets: (&[PocketDef {
    id: "cotton_underwear.front_left",
    capacity: 300.mL(),
    max_weight: 500.g(),
    rigid: false,
    ..
  }]),
  ..
};
pub const COTTON_PANTIES: &ItemDef =
  &ItemDef { id: "cotton_panties", volume: (L(1)), weight: (g(200)), wearable: true, .. };
pub const FIRE: &ItemDef =
  &ItemDef { id: "fire", volume: (mL(1)), weight: (Mass::ZERO), phase: (Phase::Gas), .. };

pub static WATER_BOTTLE: LazyLock<Item> = LazyLock::new(|| {
  let mut item: Item = PLASTIC_BOTTLE.into();
  item.pockets[0].stacks.insert_stack(WATER.item() * 500);
  item
});

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn drinking_sea_water_applies_its_thirst_penalty_without_panicking() {
    let mut player = crate::player::Player::default();
    let initial_water = player.water;

    (SEA_WATER.uses[0].on_use)(&mut player);

    assert_eq!(player.water, initial_water - 20.s());
  }
}

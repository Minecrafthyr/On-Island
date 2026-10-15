use std::sync::LazyLock;

use rand::random_range;

use super::*;
use crate::location::EnvCondition;

macro_rules! defines {
  (
    $Struct:ty;
    $($name:literal {
      $($field:ident: $value:expr),*$(,)?
    }$(.$method:ident($($args:tt)*))*)*
    $(,)?
  ) => {
    ::paste::paste! {
      $(
        pub const [<$name:upper>]: &$Struct = &$Struct {
          id: $name,
          $($field: $value),*,
          ..
        }$(.$method($($args)*))*;
      )*

    }
  };
}
defines! {
  ItemDef;
  "sand" {
    volume: 65.nL(),
    weight: 170.ug()
  }
  "biscuit" {
    volume: 100.mL(),
    weight: 20.g(),
    uses: &[UseData {
      usage: "eat",
      dur: 5.s(),
      batch: (3, 0.7),
      activity: 1.03,
      on_use: |player| {
        player.energy += 1.h();
        player.water -= 1.h();
      },
    }]
  }
  "berry" {
    volume: 6.mL(),
    weight: 3.g(),
    uses: &[UseData {
      usage: "eat",
      dur: 100.ms(),
      batch: (2, 0.5),
      activity: 1.02,
      on_use: |player| {
        player.energy += 2.mnt();
        player.water += 1.mnt();
      },
    }]
  }
  "berry_branch" {
    volume: 300.mL(),
    weight: 200.g(),
  }
  "berry_bush" {
    volume: 6.L(),
    weight: 2.kg(),
  }
  "berry_bush_harvested" {
    volume: 5.L(),
    weight: 1200.g(),
  }
  "water" {
    volume: 1.uL(),
    weight: 1.mg(),
    phase: Phase::Liquid,
    uses: &[UseData {
      usage: "drink",
      dur: 10.ms(),
      batch: (2, 0.0),
      activity: 1.01,
      on_use: |player| {
        player.water += 30.s();
      },
    }]
  }
  "sea_water" {
    volume: 1.uL(),
    weight: 1025.ug(),
    phase: Phase::Liquid,
    uses: &[UseData {
      usage: "drink",
      dur: 10.ms(),
      batch: (2, 0.0),
      activity: 1.01,
      on_use: |player| {
        player.water -= 20.s();
        todo!("salt")
      },
    }]
  }
  "plastic_bottle" {
    volume: 521.mL(),
    weight: 10.g(),
    pockets: &[PocketDef {
      id: "plastic_bottle.main",
      capacity: 500.mL(),
      max_weight: 900.g(),
      rigid: true,
      can_store_liquid: true,
      ..
    }]
  }
  "living_fish" {
    volume: 1.L(),
    weight: 1200.g(),
    gather: GatherStacks::Single(|_rd| [RAW_FISH * 1].into_iter().collect()),
    uses: &[UseData {
      usage: "eat",
      dur: 135.s(),
      batch: (1, 1.0),
      activity: 1.1,
      on_use: |player| {
        player.energy += 2.h();
        player.water += 55.mnt();
      }
    }],
    conversions: &[ConversionDef::new(EnvCondition::AlwaysTrue, 10.mnt(), || ItemStacks(vec![RAW_FISH.item() * 1]))]
  }
  "raw_fish" {
    volume: 1.L(),
    weight: 1200.g(),
    gather: GatherStacks::Single(|_rd| [RAW_FISH * 1].into_iter().collect()),
    uses: &[UseData {
      usage: "eat",
      dur: 115.s(),
      batch: (1, 1.0),
      activity: 1.06,
      on_use: |player| {
        player.energy += 2.h();
        player.water += 50.mnt();
      },
    }]
  }
  "grilled_fish" {
    volume: 900.mL(),
    weight: 1050.g(),
    uses: &[UseData {
      usage: "eat",
      dur: 105.s(),
      batch: (1, 1.0),
      activity: 1.06,
      on_use: |player| {
        player.energy += 4.h();
        player.water += 20.mnt();
      },
    }]
  }
  "tree" {
    volume: 150.L(),
    weight: 100.kg(),
    gather: GatherStacks::Single(|_rd| [
      WOOD * 2,
      BIG_BRANCH * random_range(1..2),
      BRANCH * random_range(30..60),
      STICK * random_range(0..4),
      LEAF * random_range(10..200)
   ] .into_iter().collect())
  }
  "wood" {
    volume: 50.L(),
    weight: 20.kg(),
  }
  "small_branch" {
    volume: 600.mL(),
    weight: 400.g(),
  }
  "short_stick" {
    volume: 500.mL(),
    weight: 250.g(),
  }
  "branch" {
    volume: 1300.mL(),
    weight: 1.kg(),
  }
  "stick" {
    volume: 1.L(),
    weight: 800.g(),
  }
   "big_branch" {
    volume: 2500.mL(),
    weight: 1500.g()
  }
  "long_stick" {
    volume: 2.L(),
    weight: 1.kg()
  }
  "leaf" {
    volume: 40.mL(),
    weight: 10.g()
  }
  "tree_vine" {
    volume: 500.mL(),
    weight: 300.g(),
    gather: GatherStacks::Single(|_rd| [TREE_VINE * 1].into_iter().collect())
  }
  "dry_tree_vine" {
    volume: 400.mL(),
    weight: 200.g()
  }
  "flint" {
    volume: 700.mL(),
    weight: 800.g()
  }
  "rock" {
    volume: 800.mL(),
    weight: 1.kg()
  }
  "big_rock" {
    volume: 150.L(),
    weight: 100.kg()
  }
  "vine_backpack" {
    volume: 25.L(),
    weight: 900.g(),
    pockets:
    &[PocketDef {
      id: "vine_backpack.main",
      capacity: 18.L(),
      max_weight: 10.kg(),
      rigid: false,
      ..
    }]
  }
  "vine_basket" {
    volume: L(50),
    weight: g(1500),
    pockets: &[PocketDef {
      id: "vine_basket.main",
      capacity: 48.L(),
      max_weight: 40.kg(),
      rigid: true,
      ..
    }]
  }
  "canvas_backpack" {
    volume: L(28),
    weight: g(700),
    pockets: &[PocketDef {
      id: "canvas_backpack.main",
      capacity: 20.L(),
      max_weight: 50.kg(),
      rigid: true,
      ..
    }]
  }
  "cotton_underwear" {
    volume: 5.L(),
    weight: 1200.g(),
    pockets: &[PocketDef {
      id: "cotton_underwear.front_left",
      capacity: 300.mL(),
      max_weight: 500.g(),
      rigid: false,
      ..
    }]
  }
  "cotton_panties" {
    volume: L(1),
    weight: g(200)
  }
  "fire" {
    volume: mL(1),
    weight: Mass::ZERO,
    phase: Phase::Gas
  }
}

pub static WATER_BOTTLE: LazyLock<Item> = LazyLock::new(|| {
  let mut item: Item = PLASTIC_BOTTLE.into();
  item.pockets[0].stacks.insert_stack(WATER.item() * 500);
  item
});

use std::sync::LazyLock;

use rand::random_range;

use super::*;

macro_rules! defines {
  (
    $Struct:ty,
    $( $name:literal : $f:ident ($($f_args:tt)*) $(. $method:ident ( $($args:tt)* ) )* ),*
    $(,)?
  ) => {
    ::paste::paste! {
      $(
        pub const [<$name:upper>]: &$Struct = &$Struct::$f($name, $($f_args)*)
          $(.$method($($args)*))*;
      )*

      pub const [<$Struct:upper S>]: &[&$Struct] = &[
        $( [< $name:upper >] , )*
      ];

      pub const [<$Struct:upper S_MAP>]: ::phf::Map<&'static str, &'static $Struct> =
        ::phf::phf_map! {
          $( $name => &[<$name:upper>] , )*
        };
    }
  };
}
defines! {
  ItemDef,
  "biscuit": new(mL(100), g(20)).uses(&[UseData {
    usage: "eat",
    dur: 5.s(),
    activity: 1.03,
    on_use: |player| {
    player.energy += 1.h();
    player.water -= 1.h();
  },
  }]),
  "berry": new(mL(6), g(3)).uses(&[UseData {
    usage: "eat",
    dur: 100.ms(),
    activity: 1.02,
    on_use: |player| {
    player.energy += 2.mnt();
    player.water += 1.mnt();
  },
  }]),
  "berry_branch": new(300.mL(), 200.g()),
  "berry_bush": new(6.L(), 2.kg()),
  "berry_bush_harvested": new(5.L(), 1200.g()),
  "water": new(1.mL(),1.g()).phase(Phase::Liquid).uses(&[UseData {
    usage: "drink",
    dur: 1.ms(),
    activity: 1.01,
    on_use: |player| {
    player.water += 30.s();
  },
  }]),
  "plastic_bottle": new(521.mL(), 10.g()).pockets(
    &[PocketDef::new("plastic_bottle.main", 500.mL(), 900.g(), true).can_store_liquid()]
  ),
  "raw_fish": new(1.L(), 1200.g()).gather(
    GatherStacks::Single(|_rd| [ItemStack::from_def(RAW_FISH, 1)].into())
  ).uses(&[UseData {
    usage: "eat",
    dur: 60.s(),
    activity: 1.06,
    on_use: |player| {
    player.energy += 2.h();
    player.water += 50.mnt();
  },
  }]),
  "tree": new(150.L(), 100.kg()).gather(
    GatherStacks::Single(|_rd| [
      WOOD * 2,
      STICK * random_range(0..2),
      TREE_STICK * random_range(40..60),
      LEAF * random_range(10..200)
    ].into())
  ),
  "wood": new(50.L(), 20.kg()),
  "stick": new(1.L(), 800.g()),
  "tree_stick": new(1300.mL(), 1.kg()),
  "leaf": new(40.mL(), 10.g()),
  "tree_vine": new(500.mL(), 300.g()).gather(
    GatherStacks::Single(|_rd| [TREE_VINE * 1].into())
  ),
  "dry_tree_vine": new(400.mL(), 200.g()),
  "big_rock": new(150.L(), 100.kg()),
  "flint": new(700.mL(), 800.g()),
  "rock": new(800.mL(), 1.kg()),
  "vine_backpack": new(25.L(), 900.g()).pockets(
    &[PocketDef::new("vine_backpack.main", 18.L(), 10.kg(), false)]
  ),
  "vine_basket": new(L(50), g(1500)).pockets(
    &[PocketDef::new("vine_basket.main", 48.L(), 40.kg(), true)]
  ),
  "canvas_backpack": new(L(28), g(700)).pockets(
    &[PocketDef::new("canvas_backpack.main", 20.L(), 50.kg(), true)]
  ),
  "cotton_underwear": new(5.L(), 1200.g()).pockets(
    &[PocketDef::new("cotton_underwear.front_left", 300.mL(), 500.g(), false)]
  ),
  "cotton_panties": new(L(1), g(200)),
  "fire": new(mL(1), Mass::ZERO).phase(Phase::Gas)
}

pub static WATER_BOTTLE: LazyLock<Item> = LazyLock::new(|| {
  let mut item: Item = PLASTIC_BOTTLE.into();
  item.pockets[0].stacks.insert_stack(WATER.item() * 500);
  item
});

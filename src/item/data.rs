use rand::random_range;

use crate::item::{container::PocketDef, *};

macro_rules! defines {
  (
    $Struct:ty,
    $( $name:literal : $f:ident ($($f_args:tt)*) $(. $method:ident ( $($args:tt)* ) )* ),*
    $(,)?
  ) => {
    ::paste::paste! {
      $(
        pub const [< $name:upper >]: &$Struct = &$Struct::$f($name, $($f_args)*)
          $(.$method($($args)*))*;
      )*

      pub const [< $Struct:upper S>]: &[&$Struct] = &[
        $( [< $name:upper >] , )*
      ];

      pub const [< $Struct:upper S_MAP>]: ::phf::Map<&'static str, &'static $Struct> =
        ::phf::phf_map! {
          $( $name => &[< $name:upper >] , )*
        };
    }
  };
}
defines! {
  ItemDef,
  "biscuit": new(mL(100), g(20)).use_data(UseData {
    dur: seconds(5),
    activity: 1.03,
    on_use: |player| {
    player.energy += hours(1);
    player.water -= hours(1);
  },
  }),
  "berry": new(mL(6), g(3)).use_data(UseData {
    dur: milliseconds(100),
    activity: 1.02,
    on_use: |player| {
    player.energy += minutes(2);
    player.water += minutes(1);
  },
  }),
  "berry_branch": new(mL(300), g(200)),
  "berry_bush": new(L(6), kg(2)),
  "berry_bush_harvested": new(L(5), g(1200)),
  // "water_bottle": new(mL(501), g(502)).use_data(UseData {
  //   dur: seconds(5),
  //   activity: 1.03,
  //   on_use: |player| {
  //   player.water += hours(2);
  // },
  // }),
  "water": new(mL(1),g(1)).phase(Phase::Liquid),
  "plastic_bottle": new(mL(521), g(10)).pockets(
    &[PocketDef{id:"vine_backpack.main",capacity:mL(500),max_weight:g(900),rigid:true,can_store_liquid:true,specify_items:None}]
  ),
  "raw_fish": new(L(1), g(1200)).gather(
    GatherStacks::Single(|_rd| [ItemStack::from_def(RAW_FISH, 1)].into())
  ).use_data(UseData {
    dur: seconds(60),
    activity: 1.06,
    on_use: |player| {
    player.energy += hours(2);
    player.water += minutes(50);
  },
  }),
  "tree": new(L(150), kg(100)).gather(
    GatherStacks::Single(|_rd| [
      ItemStack::from_def(WOOD, 2),
      (STICK, random_range(0..2)).into(),
      (TREE_STICK, random_range(40..60)).into(),
      (LEAF, random_range(10..200)).into()
    ].into())
  ),
  "wood": new(L(50), kg(20)),
  "stick": new(L(1), g(800)),
  "tree_stick": new(mL(1300), kg(1)),
  "leaf": new(mL(40), g(10)),
  "tree_vine": new(mL(500), g(300)).gather(
    GatherStacks::Single(|_rd| [ItemStack::from_def(TREE_VINE, 1)].into())
  ),
  "dry_tree_vine": new(mL(400), g(200)),
  "big_rock": new(L(150), kg(100)),
  "flint": new(mL(700), g(800)),
  "rock": new(mL(800), kg(1)),
  "vine_backpack": new(L(25), g(900)).pockets(
    &[PocketDef{id:"vine_backpack.main",capacity:L(18),max_weight:kg(10),rigid:false,can_store_liquid:false,specify_items:None}]
  ),
  "vine_basket": new(L(50), g(1500)).pockets(
    &[PocketDef{id:"vine_backpack.main",capacity:L(48),max_weight:kg(40),rigid:true,can_store_liquid:false,specify_items:None}]
  ),
  "canvas_backpack": new(L(28), g(700)).pockets(
    &[PocketDef{id:"canvas_backpack.main",capacity:L(20),max_weight:kg(50),rigid:true,can_store_liquid:false,specify_items:None}]
  ),
  "cotton_underwear": new(L(5), g(1200)).pockets(
    &[
      PocketDef{id:"cotton_underwear.front_left",capacity:mL(500),max_weight:g(500),rigid:true,can_store_liquid:false,specify_items:None},
      ]
  ),
  "cotton_panties": new(L(1), g(200))
}

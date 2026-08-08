use crate::{builder_method, item::UseData, units::*};
#[derive(Clone)]
pub struct ItemDef {
  pub id: &'static str,
  pub volume: Volume,
  pub weight: Mass,
  pub container_size: Volume,
  pub use_data: Option<UseData>,
}
impl ItemDef {
  builder_method!(id, &'static str);

  builder_method!(volume, Volume);

  builder_method!(weight, Mass);

  builder_method!(container_size, Volume);

  builder_method!(use_data, Option<UseData>);

  pub const fn new(id: &'static str, volume: Volume, weight: Mass) -> Self {
    Self { id, volume, weight, container_size: Default::default(), use_data: None }
  }

  pub const fn with_id(id: &'static str) -> Self {
    Self::new(id, Default::default(), Default::default())
  }
}
impl PartialEq for ItemDef {
  fn eq(&self, other: &Self) -> bool { self.id == other.id }
}
impl std::hash::Hash for ItemDef {
  fn hash<H: std::hash::Hasher>(&self, state: &mut H) { self.id.hash(state); }
}

macro_rules! defines_v2 {
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
defines_v2! {
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
"water_bottle": new(mL(501), g(502)).use_data(UseData {
  dur: seconds(5),
  activity: 1.03,
  on_use: |player| {
  player.water += hours(2);
},
}),
"raw_fish": new(L(1), g(1200)).use_data(UseData {
  dur: seconds(60),
  activity: 1.06,
  on_use: |player| {
  player.energy += hours(2);
  player.water += minutes(50);
},
}),
"wood": new(L(50), kg(20)),
"tree": new(L(150), kg(100)),
"stick": new(L(1), g(800)),
"big_rock": new(L(150), kg(100)),
"flint": new(mL(700), g(800)),
"rock": new(mL(800), kg(1)),
"tree_vine": new(mL(500), g(300)),
"dry_tree_vine": new(mL(400), g(200)),
"vine_backpack": new(L(25), kg(1)).container_size(L(18)),
"vine_basket": new(L(50), kg(2)).container_size(L(48)),
"canvas_backpack": new(L(28), g(700)).container_size(L(20)),
"cotton_underwear": new(L(5), g(1200)).container_size(mL(500)),
"cotton_panties": new(L(1), g(200))

}

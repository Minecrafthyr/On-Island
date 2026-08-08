use std::borrow::Cow;

pub struct SimpleDisplay<T>(pub T);
pub struct DetailedDisplay<T>(pub T);

pub trait NameAndDesc {
  const PREFIX: &str;
  fn get_id(&self) -> &str;
  fn name<'a>(&self) -> Cow<'a, str> { t!(format!("{}.{}.name", Self::PREFIX, self.get_id())) }
  fn description(&self) -> Cow<'_, str> {
    t!(format!("{}.{}.description", Self::PREFIX, self.get_id()))
  }
}

#[macro_export]
macro_rules! builder_method {
  ($field:ident, Option<$inner:ty>) => {
    ::paste::paste! {
      pub const fn [< $field _option >](mut self, value: Option<$inner>) -> Self {
        self.$field = value;
        self
      }
    }
    pub const fn $field(mut self, value: $inner) -> Self {
      self.$field = Some(value);
      self
    }
  };

  ($field:ident, $Fty:ty) => {
    pub const fn $field(mut self, value: $Fty) -> Self {
      self.$field = value;
      self
    }
  };
}
#[macro_export]
macro_rules! define_impl {
  (
    $Struct:ty,
    $( ($name:literal) $(. $method:ident ( $($args:tt)* ) )* ),* $(,)?
    $(,)? prefix = $prefix:ident
  ) => {
    ::paste::paste! {
      $(
        pub const [< $prefix:upper $name:upper >]: &$Struct =
          &$Struct::new($name)
          $(. $method ( $($args)* ) )*;
      )*

      pub const [< $Struct:upper S>]: &[&$Struct] = &[
        $( [< $prefix:upper $name:upper >] , )*
      ];

      pub const [< $Struct:upper S_MAP>]: ::phf::Map<&'static str, &'static $Struct> =
        ::phf::phf_map! {
          $( $name => &[< $prefix:upper $name:upper >] , )*
        };
    }
  };
  (
    $Struct:ty,
    $( ($name:literal) $(. $method:ident ( $($args:tt)* ) )* ),* $(,)?
    $(,)?
  ) => {
    ::paste::paste! {
      $(
        pub const [< $name:upper >]: &$Struct =
          &$Struct::new($name)
          $(. $method ( $($args)* ) )*;
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

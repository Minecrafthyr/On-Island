use std::borrow::Cow;

use crossterm::terminal::{disable_raw_mode, enable_raw_mode};

use super::*;

pub trait NameAndDesc {
  const PREFIX: &str;
  fn get_id(&self) -> &str;
  fn name(&self) -> Cow<'_, str> { t!(format!("{}.{}.name", Self::PREFIX, self.get_id())) }
  fn description(&self) -> Cow<'_, str> {
    t!(format!("{}.{}.description", Self::PREFIX, self.get_id()))
  }
}

pub struct RawModeGuard;

impl RawModeGuard {
  pub fn new() -> std::io::Result<Self> {
    enable_raw_mode()?;
    Ok(Self)
  }
}

impl Drop for RawModeGuard {
  fn drop(&mut self) { let _ = disable_raw_mode(); }
}
#[macro_export]
macro_rules! define {
  (
    $(#[$struct_meta:meta])*
    $vis:vis struct $Struct:ident {
      $id_vis:vis id : $IDty:ty,
      $(
        $f_vis:vis $field:ident : $Fty:ty
        $( = $field_default:expr)?
      ),* $(,)?
    },
    [
      $( ($name:literal) $(. $method:ident ( $($args:tt)* ) )* ),* $(,)?
    ]
    $(,)? prefix = $prefix:ident
  ) => {
    $(#[$struct_meta])*
    $vis struct $Struct {
      $id_vis id: $IDty,
      $($f_vis $field: $Fty,)*
    }

    impl $Struct {
      pub const fn new(id: $IDty) -> Self {
        Self {
          id,
          $($field: $crate::define!(@field_default $($field_default)?),)*
        }
      }

      // 为每个字段生成 builder 方法
      $crate::define!(@builder_method $Struct $(, $field: $Fty)*);
    }
    impl PartialEq for $Struct {
      fn eq(&self, other: &Self) -> bool { self.id == other.id }
    }

    ::paste::paste! {
      $(
        #[allow(non_upper_case_globals)]
        $vis const [< $prefix:upper $name:upper >]: &$Struct =
          &$Struct::new($name)
          $(. $method ( $($args)* ) )*;
      )*

      $vis const [< $Struct:upper S>]: &[&$Struct] = &[
        $( [< $prefix:upper $name:upper >] , )*
      ];

      $vis const [< $Struct:upper S_MAP>]: ::phf::Map<&'static str, &'static $Struct> =
        ::phf::phf_map! {
          $( $name => &[< $prefix:upper $name:upper >] , )*
        };
    }
  };

  (
    $(#[$struct_meta:meta])*
    $vis:vis struct $Struct:ident {
      $id_vis:vis id : $IDty:ty,
      $(
        $f_vis:vis $field:ident : $Fty:ty
        $( = $field_default:expr)?
      ),* $(,)?
    },
    [
      $( ($name:literal) $(. $method:ident ( $($args:tt)* ) )* ),* $(,)?
    ]
    $(,)?
  ) => {
    $(#[$struct_meta])*
    $vis struct $Struct {
      $id_vis id: $IDty,
      $($f_vis $field: $Fty,)*
    }

    impl $Struct {
      pub const fn new(id: $IDty) -> Self {
        Self {
          id,
          $($field: $crate::define!(@field_default $($field_default)?),)*
        }
      }

      // 为每个字段生成 builder 方法
      $crate::define!(@builder_method $Struct $(, $field: $Fty)*);
    }
    impl PartialEq for $Struct {
      fn eq(&self, other: &Self) -> bool { self.id == other.id }
    }
    impl std::hash::Hash for $Struct {
      fn hash<H: std::hash::Hasher>(&self, state: &mut H) { self.id.hash(state); }
    }

    ::paste::paste! {
      $(
        #[allow(non_upper_case_globals)]
        $vis const [< $name:upper >]: &$Struct =
          &$Struct::new($name)
          $(. $method ( $($args)* ) )*;
      )*

      $vis const [< $Struct:upper S>]: &[&$Struct] = &[
        $( [< $name:upper >] , )*
      ];

      $vis const [< $Struct:upper S_MAP>]: ::phf::Map<&'static str, &'static $Struct> =
        ::phf::phf_map! {
          $( $name => &[< $name:upper >] , )*
        };
    }
  };

  (@field_default $default:expr) => {
    $default
  };

  (@field_default) => {
    Default::default()
  };

  // 辅助宏：处理所有字段的 builder 方法
  (@builder_method $Struct:ident $(, $field:ident : $Fty:ty)*) => {
    $(
      $crate::define!(@impl_builder $Struct, $field, $Fty);
    )*
  };

  // 匹配 Option<T> 类型 - 生成两个方法
  (@impl_builder $Struct:ident, $field:ident, Option<$inner:ty>) => {
    // 方法1: 接受 Option<T>，直接设置
    pub const fn $field(mut self, value: Option<$inner>) -> Self {
      self.$field = value;
      self
    }

    // 方法2: 接受 T，自动包装成 Some
    ::paste::paste! {
      pub const fn [< $field:snake >] (mut self, value: $inner) -> Self {
        self.$field = Some(value);
        self
      }
    }
  };

  // 匹配非 Option 类型 - 生成普通方法
  (@impl_builder $Struct:ident, $field:ident, $Fty:ty) => {
    pub const fn $field(mut self, value: $Fty) -> Self {
      self.$field = value;
      self
    }
  };
}

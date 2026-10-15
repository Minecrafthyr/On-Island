use std::{
  cmp::{Eq, Ord, PartialEq, PartialOrd},
  ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Sub, SubAssign},
};

use paste::paste;

pub trait Zero {
  const ZERO: Self;
}

macro_rules! unit {
  (
    // $(#[$meta:meta])*
    $vis:vis struct $Type:ident($int:ty);
    $($name:ident : $factor:expr),+ $(,)?
  ) => {
    // $(#[$meta])*
    #[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
    #[derive_const(Default)]
    $vis struct $Type($int);
    impl $Type {
      $(
        #[allow(non_snake_case)]
        pub const fn $name(value: $int) -> Self {
          Self(value * $factor)
        }
        paste!{
        #[allow(non_snake_case)]
        pub const fn [<as_ $name _f64>](self) -> f64 {
          self.0 as f64 / $factor as f64
        }}
        paste!{
        #[allow(non_snake_case)]
        pub const fn [<as_ $name>](self) -> $int {
          self.0 / $factor
        }}
      )+
    }

    $(
      #[allow(non_snake_case)]
      pub const fn $name(value: $int) -> $Type {
        <$Type>::$name(value)
      }
    )+

    impl $Type {
      fn from_value_and_str(value: $int, s: &str) -> Result<$Type, String> {
        Ok(match s.to_lowercase().as_str() {
          $(stringify!($name) => $name(value),)+
          _ => return Err("Invalid unit".to_string()),
        })
      }
    }
    paste! {
    pub const trait [<To $Type>] where Self:Sized {
      $(
      #[allow(non_snake_case)]
      fn $name(self) -> $Type;
    )+
    }

    const impl [<To $Type>] for $int {
      $(fn $name(self) -> $Type {
        $Type::$name(self)
      })+
    }
    }
    impl Add for $Type {
      type Output = Self;

      fn add(self, other: Self) -> Self { Self(self.0 + other.0) }
    }
    impl AddAssign for $Type {
      fn add_assign(&mut self, other: Self) { self.0 += other.0; }
    }
    impl Sub for $Type {
      type Output = Self;

      fn sub(self, other: Self) -> Self { Self(self.0 - other.0) }
    }
    impl SubAssign for $Type {
      fn sub_assign(&mut self, other: Self) { self.0 -= other.0; }
    }
    impl Mul<$int> for $Type {
      type Output = Self;

      fn mul(self, rhs: $int) -> Self { Self(self.0 * rhs) }
    }
    impl MulAssign<$int> for $Type {
      fn mul_assign(&mut self, rhs: $int) { self.0 *= rhs; }
    }
    impl Mul<f64> for $Type {
      type Output = Self;

      fn mul(self, rhs: f64) -> Self { Self((self.0 as f64 * rhs) as $int) }
    }
    impl MulAssign<f64> for $Type {
      fn mul_assign(&mut self, rhs: f64) { self.0 = (self.0 as f64 * rhs) as $int; }
    }
    impl Div<$int> for $Type {
      type Output = Self;

      fn div(self, rhs: $int) -> Self { Self(self.0 / rhs) }
    }
    impl DivAssign<$int> for $Type {
      fn div_assign(&mut self, rhs: $int) { self.0 /= rhs; }
    }
    impl std::ops::Neg for $Type {
      type Output = Self;

      fn neg(self) -> Self { Self(self.0.wrapping_neg()) }
    }
    impl From<$Type> for $int {
      fn from(value: $Type) -> Self { value.0 }
    }
    impl From<$int> for $Type {
      fn from(value: $int) -> Self { Self(value) }
    }

    impl Zero for $Type {
      const ZERO: Self = Self::default();
    }
    impl $Type { paste! {
      $(
        pub const [<FACTOR_ $name:upper>] : $int = $factor;
        pub const [<VAL_ $name:upper>] : $int = $factor;
      )+
    }}
  };
}

const K: u64 = 1000;
const M: u64 = K * 1000;
const G: u64 = M * 1000;
const T: u64 = G * 1000;

// WARNING: `VAL_ΜL` and `VAL_ML` look alike
unit! {
  pub struct Volume(u64);
  nL: 1, uL: K, μL: K, mm3: K, mL: M, cm3: M, L: G, dm3: G, m3: T
}
unit! {
  pub struct Mass(u64);
  ug: 1, μg:1, mg: K, g: M, kg: G
}
// WARNING: `VAL_ΜM` and `VAL_MM` look alike
unit! {
  pub struct Length(u64);
  um: 1, μm: 1, mm: K, cm: M, m: M*100, km: G*100
}
// WARNING: `VAL_ΜS` and `VAL_MS` look alike
unit! {
  pub struct Duration(i64);
  us: 1, μs: 1, ms: 1000, s: 1000*1000, mnt: 1000*1000*60, h: 1000i64*1000*60*60, day: 1000i64*1000*60*60*24, week: 1000i64*1000*60*60*24*7
}
impl Duration {
  pub fn from_s_f64(secs: f64) -> Self {
    if secs.is_nan() {
      return Self(0);
    }
    let ns = secs * Duration::FACTOR_S as f64;
    let clamped = if ns >= i64::MAX as f64 {
      i64::MAX
    } else if ns <= i64::MIN as f64 {
      i64::MIN
    } else {
      ns.round() as i64
    };

    Self(clamped)
  }
}
fn fmt_unit(
  f: &mut std::fmt::Formatter<'_>, value: u64, units: &[(u64, &str)], prefix: &str,
) -> std::fmt::Result {
  if value == 0 {
    let suffix = units.last().unwrap().1;
    return write!(f, "0{}", t!(format!("{}.{}", prefix, suffix)));
  }
  for &(scale, suffix) in units {
    if value >= scale {
      let whole = value / scale;
      let rem = value % scale;
      if rem == 0 {
        return write!(f, "{}{}", whole, t!(format!("{}.{}", prefix, suffix)));
      }
      let frac_digits = scale.to_string().len() as u32 - 1;
      let frac = format!("{:0width$}", rem, width = frac_digits as usize);
      let frac = frac.trim_end_matches('0');
      return write!(f, "{}.{}{}", whole, frac, t!(format!("{}.{}", prefix, suffix)));
    }
  }
  unreachable!()
}
impl std::fmt::Display for Volume {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    fmt_unit(
      f,
      self.0,
      &[
        (Volume::FACTOR_M3, "m3"),
        (Volume::FACTOR_L, "L"),
        (Volume::FACTOR_ML, "mL"),
        (Volume::FACTOR_ΜL, "μL"),
        (Volume::FACTOR_UL, "nL"),
      ],
      "unit.vol",
    )
  }
}

impl std::fmt::Display for Mass {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    fmt_unit(
      f,
      self.0,
      &[
        (Mass::FACTOR_KG, "kg"),
        (Mass::FACTOR_G, "g"),
        (Mass::FACTOR_MG, "mg"),
        (Mass::FACTOR_ΜG, "μg"),
      ],
      "unit.mass",
    )
  }
}

impl std::fmt::Display for Length {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    fmt_unit(
      f,
      self.0,
      &[
        (Length::FACTOR_KM, "km"),
        (Length::FACTOR_M, "m"),
        (Length::FACTOR_CM, "cm"),
        (Length::FACTOR_ΜM, "μm"),
      ],
      "unit.len",
    )
  }
}

impl std::fmt::Display for Duration {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    let ns = self.0;
    if ns == 0 {
      return write!(f, "0{}", t!("unit.dur.ns"));
    }
    let abs = ns.abs();
    if ns < 0 {
      write!(f, "-")?;
    }
    for (scale, suffix) in [
      (Duration::FACTOR_DAY, "d"),
      (Duration::FACTOR_H, "h"),
      (Duration::FACTOR_MNT, "m"),
      (Duration::FACTOR_S, "s"),
      (Duration::FACTOR_MS, "ms"),
      (Duration::FACTOR_ΜS, "µs"),
    ] {
      if abs >= scale {
        let whole = abs / scale;
        let rem = abs % scale;
        if rem == 0 {
          return write!(f, "{}{}", whole, t!(format!("unit.dur.{suffix}")));
        }
        let frac_digits = scale.to_string().len() as u32 - 1;
        let frac = format!("{:0width$}", rem, width = frac_digits as usize);
        let frac = frac.trim_end_matches('0');
        return write!(f, "{}.{}{}", whole, frac, t!(format!("unit.dur.{suffix}")));
      }
    }
    unreachable!()
  }
}

fn parse_units<A, T, F, G>(s: &str, mut acc: A, parse_num: F, apply: G) -> Result<A, String>
where
  T: Copy,
  F: Fn(&str) -> Result<T, String>,
  G: Fn(A, T, &str) -> Result<A, String>,
{
  let bytes = s.as_bytes();
  let len = bytes.len();
  let mut i = 0usize;

  while i < len {
    while i < len && bytes[i].is_ascii_whitespace() {
      i += 1;
    }
    if i >= len {
      break;
    }

    let num_start = i;
    if bytes[i] == b'-' || bytes[i] == b'+' {
      i += 1;
    }
    let digits_start = i;
    while i < len && bytes[i].is_ascii_digit() {
      i += 1;
    }
    if i == digits_start {
      return Err(format!("Expected number at position {num_start}"));
    }
    let num_text = &s[num_start..i];
    let value = parse_num(num_text)?;

    while i < len && bytes[i].is_ascii_whitespace() {
      i += 1;
    }

    let unit_start = i;
    while i < len
      && !bytes[i].is_ascii_whitespace()
      && !bytes[i].is_ascii_digit()
      && bytes[i] != b'-'
      && bytes[i] != b'+'
    {
      i += 1;
    }
    if i == unit_start {
      return Err(format!("Expected unit at position {unit_start}"));
    }
    let unit = &s[unit_start..i];

    acc = apply(acc, value, unit)?;
  }

  Ok(acc)
}
impl std::str::FromStr for Duration {
  type Err = String;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    parse_units(
      s,
      Duration::default(),
      |t| t.parse::<i64>().map_err(|e| e.to_string()),
      |acc, value, unit| Ok(acc + Duration::from_value_and_str(value, unit)?),
    )
  }
}
macro_rules! impl_from_str_unsigned {
  ($Type:ty) => {
    impl std::str::FromStr for $Type {
      type Err = String;

      fn from_str(s: &str) -> Result<Self, Self::Err> {
        parse_units(
          s,
          <$Type>::default(),
          |t| t.parse::<u64>().map_err(|e| e.to_string()),
          |acc, value, unit| Ok(acc + <$Type>::from_value_and_str(value, unit)?),
        )
      }
    }
  };
}

impl_from_str_unsigned!(Volume);
impl_from_str_unsigned!(Mass);
impl_from_str_unsigned!(Length);

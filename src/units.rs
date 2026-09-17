use std::{
  cmp::{Eq, Ord, PartialEq, PartialOrd},
  ops::{Add, AddAssign, Deref, Div, DivAssign, Mul, MulAssign, Sub, SubAssign},
};

use paste::paste;
use time::Duration;

pub trait Zero {
  const ZERO: Self;
}

macro_rules! impl_constructors {
  ($Type:ident, $($name:ident : $factor:expr),+ $(,)?) => {
    impl $Type {
      $(
        #[allow(non_snake_case)]
        pub const fn $name(value: u64) -> Self {
          Self(value * $factor)
        }
        paste!{
        #[allow(non_snake_case)]
        pub const fn [<as_ $name _f64>](self) -> f64 {
          self.0 as f64 / $factor as f64
        }}
      )+
    }

    $(
      #[allow(non_snake_case)]
      pub const fn $name(value: u64) -> $Type {
        $Type::$name(value)
      }
    )+

    impl $Type {
      fn from_value_and_str(value: u64, s: &str) -> Result<$Type, String> {
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

    const impl [<To $Type>] for u64 {
      $(fn $name(self) -> $Type {
        $Type::$name(self)
      })+
    }
    }

    // 集成 FromStr
    impl std::str::FromStr for $Type {
      type Err = String;

      fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut result = $Type::default();
        let mut last_idx = 0;
        let mut value_buf: Option<u64> = None;
        for (i, c) in s.chars().enumerate() {
          match c {
            '0'..='9' => {}
            ' ' | 'a'..='z' | 'A'..='Z' if value_buf.is_none() => {
              value_buf = Some(
                s[last_idx..i].parse().map_err(|e: std::num::ParseIntError| e.to_string())?,
              );
              last_idx = i;
            }
            ' ' if let Some(value) = value_buf => {
              result += Self::from_value_and_str(value, &s[last_idx..i])?;
              value_buf = None;
            }
            _ => return Err("Invalid char".to_string()),
          }
        }
        if let Some(value) = value_buf {
          result += Self::from_value_and_str(value, &s[last_idx..])?;
        }
        Ok(result)
      }
    }
  };
}

macro_rules! impl_traits {
  ($Type:ty) => {
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
    impl Mul<u64> for $Type {
      type Output = Self;

      fn mul(self, rhs: u64) -> Self { Self(self.0 * rhs) }
    }
    impl MulAssign<u64> for $Type {
      fn mul_assign(&mut self, rhs: u64) { self.0 *= rhs; }
    }
    impl Mul<f64> for $Type {
      type Output = Self;

      fn mul(self, rhs: f64) -> Self { Self((self.0 as f64 * rhs) as u64) }
    }
    impl MulAssign<f64> for $Type {
      fn mul_assign(&mut self, rhs: f64) { self.0 = (self.0 as f64 * rhs) as u64; }
    }
    impl Div<u64> for $Type {
      type Output = Self;

      fn div(self, rhs: u64) -> Self { Self(self.0 / rhs) }
    }
    impl DivAssign<u64> for $Type {
      fn div_assign(&mut self, rhs: u64) { self.0 /= rhs; }
    }
    impl std::ops::Neg for $Type {
      type Output = Self;

      fn neg(self) -> Self { Self(self.0.wrapping_neg()) }
    }
    impl From<$Type> for u64 {
      fn from(value: $Type) -> Self { value.0 }
    }
    impl From<u64> for $Type {
      fn from(value: u64) -> Self { Self(value) }
    }
    impl std::fmt::Display for $Type {
      fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { write!(f, "{}", self.0) }
    }
    impl Zero for $Type {
      const ZERO: Self = Self::default();
    }
  };
}

const K: u64 = 1000;
const M: u64 = K * 1000;
const B: u64 = M * 1000;
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[derive_const(Default)]
pub struct Volume(u64);
impl_traits!(Volume);

impl_constructors!(Volume, uL: 1, mL: K, cm3: K , L: M, m3: B);

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[derive_const(Default)]
pub struct Mass(u64);
impl_traits!(Mass);
impl_constructors!(Mass, ug: 1, mg: K, g: M, kg: B);

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[derive_const(Default)]
pub struct Length(u64);
impl_traits!(Length);
impl_constructors!(Length, um: 1, mm: K, cm: 10*K, m: M, km: B);

// #[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
// #[derive_const(Default)]
// pub struct Duration(u64);
// impl_traits!(Duration);
// const US_PER_MS: u64 = 1000;
// const US_PER_S: u64 = US_PER_MS * 1000;
// const US_PER_M: u64 = US_PER_S * 60;
// const US_PER_H: u64 = US_PER_M * 60;
// impl_constructors!(Duration, us: 1, ms: US_PER_MS, s: US_PER_S, minutes: US_PER_M, hours: US_PER_H);

pub const trait ToDuration {
  fn milliseconds(self) -> Duration;
  fn seconds(self) -> Duration;
  fn minutes(self) -> Duration;
  fn hours(self) -> Duration;
  fn days(self) -> Duration;
  fn weeks(self) -> Duration;
}
const impl ToDuration for i64 {
  #[inline]
  fn milliseconds(self) -> Duration { Duration::milliseconds(self) }

  #[inline]
  fn seconds(self) -> Duration { Duration::seconds(self) }

  #[inline]
  fn minutes(self) -> Duration { Duration::minutes(self) }

  #[inline]
  fn hours(self) -> Duration { Duration::hours(self) }

  #[inline]
  fn days(self) -> Duration { Duration::days(self) }

  #[inline]
  fn weeks(self) -> Duration { Duration::weeks(self) }
}

// #[inline]
// pub const fn milliseconds(milliseconds: i64) -> Duration { Duration::milliseconds(milliseconds) }
// #[inline]
// pub const fn seconds(seconds: i64) -> Duration { Duration::seconds(seconds) }
// #[inline]
// pub const fn minutes(minutes: i64) -> Duration { Duration::minutes(minutes) }
// #[inline]
// pub const fn hours(hours: i64) -> Duration { Duration::hours(hours) }
// #[inline]
// pub const fn days(days: i64) -> Duration { Duration::days(days) }
// #[inline]
// pub const fn weeks(weeks: i64) -> Duration { Duration::weeks(weeks) }

#[derive(Default)]
pub struct DurationDisplay(pub Duration);

impl Deref for DurationDisplay {
  type Target = Duration;

  fn deref(&self) -> &Self::Target { &self.0 }
}

impl std::fmt::Display for DurationDisplay {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    use time::unit::*;
    if self.is_negative() {
      f.write_str("-")?;
    }
    if let Some(_precision) = f.precision() {
      if self.is_zero() {
        return (0.).fmt(f).and_then(|_| f.write_str("s"));
      }

      macro_rules! item {
        ($name:literal, $value:expr) => {
          let value = $value;
          if value >= 1.0 {
            return value.fmt(f).and_then(|_| write!(f, "{}", t!($name)));
          }
        };
      }

      let seconds = self.unsigned_abs().as_secs_f64();
      item!("ui.dur.d", seconds / Second::per_t::<f64>(Day));
      item!("ui.dur.h", seconds / Second::per_t::<f64>(Hour));
      item!("ui.dur.m", seconds / Second::per_t::<f64>(Minute));
      item!("ui.dur.s", seconds);
      item!("ui.dur.ms", seconds * Millisecond::per_t::<f64>(Second));
      item!("ui.dur.µs", seconds * Microsecond::per_t::<f64>(Second));
      item!("ui.dur.ns", seconds * Nanosecond::per_t::<f64>(Second));
    } else {
      if self.is_zero() {
        return f.write_str("0秒");
      }
      macro_rules! item {
        ($name:literal, $value:expr) => {
          match $value {
          0 => Ok(()),
          value => value.fmt(f).and_then(|_| f.write_str(&t!($name))),
          }
        };
      }
      let seconds = self.whole_seconds().unsigned_abs();
      let nanoseconds = self.subsec_nanoseconds().unsigned_abs();
      item!("ui.dur.d", seconds / Second::per_t::<u64>(Day))?;
      item!("ui.dur.h", seconds / Second::per_t::<u64>(Hour) % Hour::per_t::<u64>(Day))?;
      item!("ui.dur.m", seconds / Second::per_t::<u64>(Minute) % Minute::per_t::<u64>(Hour))?;
      item!("ui.dur.s", seconds % Second::per_t::<u64>(Minute))?;
      item!("ui.dur.ms", nanoseconds / Nanosecond::per_t::<u32>(Millisecond))?;
      item!(
        "ui.dur.µs",
        nanoseconds / Nanosecond::per_t::<u32>(Microsecond)
          % Microsecond::per_t::<u32>(Millisecond)
      )?;
      item!("ns", nanoseconds % Nanosecond::per_t::<u32>(Microsecond))?;
    }

    Ok(())
  }
}

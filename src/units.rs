use std::{
  cmp::{Eq, Ord, PartialEq, PartialOrd},
  ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Sub, SubAssign},
};

use paste::paste;
use time::Duration;
pub trait Zero: const Default {
  const ZERO: Self = Self::default();
}

macro_rules! impl_constructors {
    ($type:ident, $($name:ident : $factor:expr),+ $(,)?) => {
        impl $type {
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
            pub const fn $name(value: u64) -> $type {
                $type::$name(value)
            }
        )+

        impl $type  {
        fn from_value_and_str(value: u64, s: &str) -> Result<$type, String> {
            Ok(match s.to_lowercase().as_str() {
                $(stringify!($name) => $name(value),)+
                _ => return Err("Invalid unit".to_string()),
            })
        }

        }

        // 集成 FromStr
        impl std::str::FromStr for $type {
            type Err = String;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                let mut result = $type::default();
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
  ($type:ty) => {
    // 加法
    impl Add for $type {
      type Output = Self;

      fn add(self, other: Self) -> Self { Self(self.0 + other.0) }
    }

    impl AddAssign for $type {
      fn add_assign(&mut self, other: Self) { self.0 += other.0; }
    }

    // 减法
    impl Sub for $type {
      type Output = Self;

      fn sub(self, other: Self) -> Self { Self(self.0 - other.0) }
    }

    impl SubAssign for $type {
      fn sub_assign(&mut self, other: Self) { self.0 -= other.0; }
    }

    impl Mul<u64> for $type {
      type Output = Self;

      fn mul(self, rhs: u64) -> Self { Self(self.0 * rhs) }
    }

    impl MulAssign<u64> for $type {
      fn mul_assign(&mut self, rhs: u64) { self.0 *= rhs; }
    }
    impl Mul<f64> for $type {
      type Output = Self;

      fn mul(self, rhs: f64) -> Self { Self((self.0 as f64 * rhs) as u64) }
    }

    impl MulAssign<f64> for $type {
      fn mul_assign(&mut self, rhs: f64) { self.0 = (self.0 as f64 * rhs) as u64; }
    }

    impl Div<u64> for $type {
      type Output = Self;

      fn div(self, rhs: u64) -> Self { Self(self.0 / rhs) }
    }

    impl DivAssign<u64> for $type {
      fn div_assign(&mut self, rhs: u64) { self.0 /= rhs; }
    }

    // 负运算符（可选）
    impl std::ops::Neg for $type {
      type Output = Self;

      fn neg(self) -> Self { Self(self.0.wrapping_neg()) }
    }
    impl From<$type> for u64 {
      fn from(value: $type) -> Self { value.0 }
    }

    impl From<u64> for $type {
      fn from(value: u64) -> Self { Self(value) }
    }

    impl std::fmt::Display for $type {
      fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { write!(f, "{}", self.0) }
    }
    impl Zero for $type {}
  };
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[derive_const(Default)]
pub struct Volume(u64);
impl_traits!(Volume);

const UL_PER_ML: u64 = 1000;
const UL_PER_L: u64 = UL_PER_ML * 1000;
const UL_PER_M3: u64 = UL_PER_L * 1000;
impl_constructors!(Volume, uL: 1, mL: UL_PER_ML, cm3: UL_PER_ML, L: UL_PER_L, m3: UL_PER_M3);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[derive_const(Default)]
pub struct Mass(u64);
impl_traits!(Mass);
const UG_PER_MG: u64 = 1000;
const UG_PER_G: u64 = UG_PER_MG * 1000;
const UG_PER_KG: u64 = UG_PER_G * 1000;
impl_constructors!(Mass, ug: 1, mg: UG_PER_MG, g: UG_PER_G, kg: UG_PER_KG);

// #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
// #[derive_const(Default)]
// pub struct Duration(u64);
// impl_traits!(Duration);
// const US_PER_MS: u64 = 1000;
// const US_PER_S: u64 = US_PER_MS * 1000;
// const US_PER_M: u64 = US_PER_S * 60;
// const US_PER_H: u64 = US_PER_M * 60;
// impl_constructors!(Duration, us: 1, ms: US_PER_MS, s: US_PER_S, minutes: US_PER_M, hours: US_PER_H);

#[inline]
pub const fn seconds(seconds: i64) -> Duration { Duration::seconds(seconds) }
#[inline]
pub const fn minutes(minutes: i64) -> Duration { Duration::minutes(minutes) }
#[inline]
pub const fn hours(hours: i64) -> Duration { Duration::hours(hours) }
#[inline]
pub const fn days(days: i64) -> Duration { Duration::days(days) }

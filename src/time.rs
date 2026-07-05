use std::{
    fmt::{Debug, Display},
    ops::*,
};

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Time(pub u64);
const S: u64 = 1000;
const M: u64 = S * 60;
const H: u64 = M * 60;
const D: u64 = H * 24;
impl Time {
    pub const fn new() -> Self { Self(0) }

    pub const fn s(seconds: u64) -> Self { Self(seconds * S) }

    pub const fn m(minutes: u64) -> Self { Self(minutes * M) }

    pub const fn h(hours: u64) -> Self { Self(hours * H) }

    pub const fn d(days: u64) -> Self { Self(days * D) }

    pub const fn as_f64(self) -> f64 { self.0 as f64 }
}
impl Display for Time {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut t = self.0;
        let h;
        if t > H {
            h = t / H;
            t -= h * H;
            write!(f, "{h}h")?;
        }
        let m;
        if t > M {
            m = t / M;
            t -= m * M;
            write!(f, "{m}m")?;
        }
        let s;
        if t > S {
            s = t / S;
            write!(f, "{s}s")?;
        }
        Ok(())
    }
}
impl Debug for Time {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut t = self.0;
        let h;
        if t > H {
            h = t / H;
            t -= h * H;
            write!(f, "{h}h")?;
        }
        let m;
        if t > M {
            m = t / M;
            t -= m * M;
            write!(f, "{m}m")?;
        }
        let s;
        if t > S {
            s = t / S;
            t -= s * S;
            write!(f, "{s}s")?;
        }
        write!(f, "{t}ms")?;
        Ok(())
    }
}
impl Into<f64> for Time {
    fn into(self) -> f64 { self.0 as f64 }
}
impl Add for Time {
    type Output = Self;

    fn add(self, rhs: Self) -> Self { Time(self.0 + rhs.0) }
}
impl AddAssign for Time {
    fn add_assign(&mut self, rhs: Self) { self.0 += rhs.0 }
}

impl Sub for Time {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self { Time(self.0 - rhs.0) }
}
impl SubAssign for Time {
    fn sub_assign(&mut self, rhs: Self) { self.0 -= rhs.0 }
}

impl Rem for Time {
    type Output = Self;

    fn rem(self, rhs: Self) -> Self { Time(self.0 % rhs.0) }
}
impl RemAssign for Time {
    fn rem_assign(&mut self, rhs: Self) { self.0 %= rhs.0 }
}

pub mod offset {
    use super::*;
    const S: i64 = 1000;
    const M: i64 = S * 60;
    const H: i64 = M * 60;
    const D: i64 = H * 24;
    #[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
    pub struct TimeOffset(pub i64);

    impl TimeOffset {
        pub const fn new() -> Self { Self(0) }

        pub const fn s(seconds: i64) -> Self { Self(seconds * S) }

        pub const fn m(minutes: i64) -> Self { Self(minutes * M) }

        pub const fn h(hours: i64) -> Self { Self(hours * H) }

        pub const fn d(days: i64) -> Self { Self(days * D) }

        pub const fn as_f64(self) -> f64 { self.0 as f64 }

        pub const fn abs(self) -> Self { Self(self.0.abs()) }

        pub const fn is_positive(self) -> bool { self.0 > 0 }

        pub const fn is_negative(self) -> bool { self.0 < 0 }

        pub const fn is_zero(self) -> bool { self.0 == 0 }
    }

    impl Display for TimeOffset {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            let mut t = self.0.abs();
            let sign = if self.0 < 0 { "-" } else { "" };

            let h;
            if t >= H {
                h = t / H;
                t -= h * H;
                write!(f, "{sign}{h}h")?;
            }
            let m;
            if t >= M {
                m = t / M;
                t -= m * M;
                write!(f, "{m}m")?;
            }
            let s;
            if t >= S {
                s = t / S;
                t -= s * S;
                write!(f, "{s}s")?;
            }
            if t == 0 && self.0 == 0 {
                write!(f, "0s")?;
            }
            Ok(())
        }
    }

    impl Debug for TimeOffset {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            let mut t = self.0.abs();
            let sign = if self.0 < 0 { "-" } else { "" };

            let h;
            if t >= H {
                h = t / H;
                t -= h * H;
                write!(f, "{sign}{h}h")?;
            }
            let m;
            if t >= M {
                m = t / M;
                t -= m * M;
                write!(f, "{m}m")?;
            }
            let s;
            if t >= S {
                s = t / S;
                t -= s * S;
                write!(f, "{s}s")?;
            }
            write!(f, "{t}ms")?;
            Ok(())
        }
    }

    impl From<TimeOffset> for f64 {
        fn from(val: TimeOffset) -> Self { val.0 as f64 }
    }

    impl From<Time> for TimeOffset {
        fn from(time: Time) -> Self { TimeOffset(time.0 as i64) }
    }

    impl Neg for TimeOffset {
        type Output = Self;

        fn neg(self) -> Self { Self(-self.0) }
    }

    impl Add for TimeOffset {
        type Output = Self;

        fn add(self, rhs: Self) -> Self { TimeOffset(self.0 + rhs.0) }
    }

    impl AddAssign for TimeOffset {
        fn add_assign(&mut self, rhs: Self) { self.0 += rhs.0; }
    }

    impl Sub for TimeOffset {
        type Output = Self;

        fn sub(self, rhs: Self) -> Self { TimeOffset(self.0 - rhs.0) }
    }

    impl SubAssign for TimeOffset {
        fn sub_assign(&mut self, rhs: Self) { self.0 -= rhs.0; }
    }

    // Time 与 TimeOffset 的交互运算
    impl Add<TimeOffset> for Time {
        type Output = Time;

        fn add(self, rhs: TimeOffset) -> Self::Output {
            // 使用 checked_add 避免溢出
            let result = self.0 as i64 + rhs.0;
            if result < 0 {
                panic!("Time addition underflow: result would be negative");
            }
            Time(result as u64)
        }
    }

    impl Add<Time> for TimeOffset {
        type Output = Time;

        fn add(self, rhs: Time) -> Self::Output { rhs + self }
    }

    impl AddAssign<TimeOffset> for Time {
        fn add_assign(&mut self, rhs: TimeOffset) {
            let result = self.0 as i64 + rhs.0;
            if result < 0 {
                panic!("Time addition underflow: result would be negative");
            }
            self.0 = result as u64;
        }
    }

    impl Sub<TimeOffset> for Time {
        type Output = Time;

        fn sub(self, rhs: TimeOffset) -> Self::Output {
            let result = self.0 as i64 - rhs.0;
            if result < 0 {
                panic!("Time subtraction underflow: result would be negative");
            }
            Time(result as u64)
        }
    }

    impl Sub<Time> for TimeOffset {
        type Output = TimeOffset;

        fn sub(self, rhs: Time) -> Self::Output { TimeOffset(self.0 - rhs.0 as i64) }
    }

    impl SubAssign<TimeOffset> for Time {
        fn sub_assign(&mut self, rhs: TimeOffset) {
            let result = self.0 as i64 - rhs.0;
            if result < 0 {
                panic!("Time subtraction underflow: result would be negative");
            }
            self.0 = result as u64;
        }
    }

    impl Rem<TimeOffset> for TimeOffset {
        type Output = Self;

        fn rem(self, rhs: Self) -> Self { TimeOffset(self.0 % rhs.0) }
    }

    impl RemAssign<TimeOffset> for TimeOffset {
        fn rem_assign(&mut self, rhs: Self) { self.0 %= rhs.0; }
    }
}

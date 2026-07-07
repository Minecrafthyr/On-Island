use std::{cell::UnsafeCell, sync::LazyLock};

pub struct UnsafeLocker<T>(UnsafeCell<LazyLock<T>>);
unsafe impl<T> Sync for UnsafeLocker<T> {}
impl<T> UnsafeLocker<T> {
    pub const fn new(f: fn() -> T) -> Self { Self(UnsafeCell::new(LazyLock::new(f))) }

    pub fn get_mut(&self) -> &mut T { unsafe { &mut *self.0.get() } }
}

type StdoutLock = std::io::StdoutLock<'static>;
static STDOUT: UnsafeLocker<StdoutLock> = UnsafeLocker::new(|| std::io::stdout().lock());

pub fn stdout() -> &'static mut StdoutLock { STDOUT.get_mut() }

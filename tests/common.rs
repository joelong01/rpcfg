use std::sync::atomic::{AtomicBool, Ordering};
use std::cell::Cell;
use std::panic;

pub static GLOBAL_TEST_MODE: AtomicBool = AtomicBool::new(false);

thread_local! {
    pub static THREAD_TEST_MODE: Cell<bool> = Cell::new(false);
}

pub struct TestModeGuard;

impl TestModeGuard {
    pub fn new() -> Self {
        THREAD_TEST_MODE.with(|mode| mode.set(true));
        GLOBAL_TEST_MODE.store(true, Ordering::SeqCst);
        TestModeGuard
    }
}

impl Drop for TestModeGuard {
    fn drop(&mut self) {
        THREAD_TEST_MODE.with(|mode| mode.set(false));
    }
}

pub fn is_test_mode() -> bool {
    THREAD_TEST_MODE.with(|mode| mode.get()) || GLOBAL_TEST_MODE.load(Ordering::SeqCst)
}

pub fn run_test<T>(test: T) -> ()
where
    T: FnOnce() + panic::UnwindSafe,
{
    let guard = TestModeGuard::new();
    let result = panic::catch_unwind(test);
    drop(guard);
    assert!(result.is_ok(), "Test panicked");
}

#[macro_export]
macro_rules! safe_test {
    ($(#[$meta:meta])* $name:ident, $body:expr) => {
        $(#[$meta])*
        #[test]
        fn $name() {
            $crate::tests::common::run_test(|| $body);
        }
    };
}

// Add other common test utilities here

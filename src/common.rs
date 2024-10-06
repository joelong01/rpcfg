use std::sync::atomic::{AtomicBool, Ordering};
use std::cell::Cell;
use std::panic;
use std::sync::Once;
use tracing::Level;
use tracing_subscriber::FmtSubscriber;

pub static GLOBAL_TEST_MODE: AtomicBool = AtomicBool::new(false);

thread_local! {
    pub static THREAD_TEST_MODE: Cell<bool> = Cell::new(false);
}

static INIT: Once = Once::new();

pub fn initialize_subscriber() {
    INIT.call_once(|| {
        let subscriber = FmtSubscriber::builder()
            .with_max_level(Level::DEBUG)
            .finish();
        tracing::subscriber::set_global_default(subscriber)
            .expect("setting default subscriber failed");
    });
}

pub struct TestModeGuard;

impl TestModeGuard {
    pub fn new() -> Self {
        initialize_subscriber();
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

pub fn run_test<T>(test: T) -> Result<(), Box<dyn std::any::Any + Send>>
where
    T: FnOnce() -> Result<(), anyhow::Error> + panic::UnwindSafe,
{
    let guard = TestModeGuard::new();
    let result = panic::catch_unwind(|| {
        test().map_err(|e| panic::panic_any(e))
    });
    drop(guard);
    result.unwrap_or_else(|e| Err(e))
}

#[macro_export]
macro_rules! safe_test {
    ($(#[$meta:meta])* $name:ident, $body:expr) => {
        $(#[$meta])*
        #[test]
        fn $name() {
            $crate::common::run_test(|| $body);
        }
    };
}

// Add other common test utilities here

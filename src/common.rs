use std::panic;
use std::sync::Once;
use tracing::Level;
use tracing_subscriber::FmtSubscriber;

static INIT: Once = Once::new();

/// Initializes the tracing subscriber for logging.
///
/// This function sets up the global tracing subscriber with a maximum log level of DEBUG.
/// It uses a `Once` guard to ensure that the subscriber is only initialized once,
/// even if the function is called multiple times.
///
/// # Panics
///
/// Panics if setting the global default subscriber fails.
pub fn initialize_subscriber() {
    INIT.call_once(|| {
        let subscriber = FmtSubscriber::builder()
            .with_max_level(Level::DEBUG)
            .finish();
        tracing::subscriber::set_global_default(subscriber)
            .expect("setting default subscriber failed");
    });
}

/// A guard struct that ensures the tracing subscriber is initialized for tests.
///
/// When created, this guard initializes the tracing subscriber.
/// It doesn't do anything when dropped, as the subscriber initialization is a global operation.
pub struct SubscriberGuard;

impl SubscriberGuard {
    /// Creates a new SubscriberGuard, initializing the tracing subscriber.
    ///
    /// # Returns
    ///
    /// A new instance of SubscriberGuard.
    ///
    /// # Example
    ///
    /// ```
    /// use rp::common::SubscriberGuard;
    ///
    /// let _guard = SubscriberGuard::new();
    /// // The subscriber is now initialized
    /// ```
    pub fn new() -> Self {
        initialize_subscriber();
        SubscriberGuard
    }
}

/// Runs a test function with proper setup and error handling.
///
/// This function creates a SubscriberGuard to ensure logging is set up,
/// executes the provided test function, and properly handles any panics or errors.
///
/// # Type Parameters
///
/// * `T`: A function that returns a Result and can be unwound safely in case of a panic.
///
/// # Arguments
///
/// * `test`: The test function to run.
///
/// # Returns
///
/// Returns `Ok(())` if the test passes, or an error if the test fails or panics.
///
/// # Example
///
/// ```
/// use rp::common::run_test;
/// use anyhow::Result;
///
/// fn my_test() -> Result<()> {
///     // Test code here
///     Ok(())
/// }
///
/// let result = run_test(my_test);
/// assert!(result.is_ok());
/// ```
pub fn run_test<T>(test: T) -> Result<(), Box<dyn std::any::Any + Send>>
where
    T: FnOnce() -> Result<(), anyhow::Error> + panic::UnwindSafe,
{
    let guard = SubscriberGuard::new();
    let result = panic::catch_unwind(|| {
        test().map_err(|e| panic::panic_any(e))
    });
    drop(guard);
    result.unwrap_or_else(|e| Err(e))
}

/// A macro for defining safe test functions.
///
/// This macro wraps a test body in the `run_test` function, providing proper setup and error handling.
///
/// # Arguments
///
/// * `$name`: The name of the test function.
/// * `$body`: The body of the test function.
///
/// # Example
///
/// ```
/// use rp::safe_test;
/// use anyhow::Result;
///
/// safe_test!(my_test, {
///     // Test code here
///     Ok(())
/// });
/// ```
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

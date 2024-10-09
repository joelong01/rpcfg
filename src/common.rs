use std::{backtrace, panic};
use std::sync::Once;
use tracing::Level;
use tracing_subscriber::FmtSubscriber;
use backtrace::Backtrace;

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
    /// use rpcfg::common::SubscriberGuard;
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
/// use rpcfg::common::run_test;
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
pub fn run_test<T>(test: T) -> Result<(), String>
where
    T: FnOnce() -> Result<(), anyhow::Error> + panic::UnwindSafe,
{
    let guard = SubscriberGuard::new();
    let result: Result<Result<(), anyhow::Error>, Box<dyn std::any::Any + Send>> = panic::catch_unwind(|| {
        test().map_err(|e| {
            let bt = Backtrace::capture();
            let formatted_backtrace = format!("{:?}", bt);
            let formatted_lines: Vec<_> = formatted_backtrace
                .lines()
                .filter(|line| line.contains("src"))
                .map(|line| line.trim())
                .collect();
            anyhow::anyhow!(
                "Test failed: {}\n\nRelevant backtrace:\n{}",
                e,
                formatted_lines.join("\n")
            )
        })
    });
    drop(guard);

    match result {
        Ok(Ok(())) => Ok(()),
        Ok(Err(e)) => Err(e.to_string()),
        Err(e) => {
            if let Some(s) = e.downcast_ref::<String>() {
                Err(s.clone())
            } else if let Some(s) = e.downcast_ref::<&str>() {
                Err(s.to_string())
            } else {
                Err("Unknown panic occurred".to_string())
            }
        }
    }
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
/// use rpcfg::safe_test;
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
            match $crate::common::run_test(|| $body) {
                Ok(_) => (),
                Err(e) => panic!("Test failed: {}", e),
            }
        }
    };
}

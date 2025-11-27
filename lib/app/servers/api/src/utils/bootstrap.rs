use env_logger::{Builder, Env};
use std::sync::Once;

/// Initialize env_logger once for the entire process.
///
/// Web and test binaries can call this helper before invoking `run()` to make
/// sure structured logs are emitted consistently.
pub fn init_logging() {
    static INIT: Once = Once::new();
    INIT.call_once(|| {
        let _ = Builder::from_env(Env::default().default_filter_or("info"))
            .format_timestamp_millis()
            .try_init();
    });
}

use std::sync::atomic::{AtomicBool, Ordering};

static DEBUG_ENABLED: AtomicBool = AtomicBool::new(false);

pub fn init(cli_enabled: bool) {
    DEBUG_ENABLED.store(cli_enabled || env_enabled(), Ordering::Relaxed);
}

pub fn enabled() -> bool {
    DEBUG_ENABLED.load(Ordering::Relaxed)
}

pub fn log(message: impl AsRef<str>) {
    if enabled() {
        eprintln!("[ccx:debug] {}", message.as_ref());
    }
}

fn env_enabled() -> bool {
    std::env::var("CCX_DEBUG")
        .map(|value| matches!(value.as_str(), "1" | "true" | "TRUE" | "yes" | "YES"))
        .unwrap_or(false)
}

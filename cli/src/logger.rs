use env_logger::Env;
use log::trace;

pub fn init_logging() {
    const MEMBERS: [&str; 7] = [
        "calendar",
        "cli",
        "config",
        "fs_cache",
        "time_out",
        "weather",
        "headless_chrome",
    ];
    let filters = MEMBERS.join("=trace,");
    let env = Env::default()
        .filter_or("DAILY_BUGLE_LOG", format!("{}=debug", filters))
        .write_style_or("DAILY_BUGLE_LOG_STYLE", "always");

    env_logger::Builder::from_env(env).init();

    trace!("Logging Initialized");
}

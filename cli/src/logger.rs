use env_logger::Env;
use log::trace;

pub fn init_logging() {
    const MEMBERS: [&str; 5] = ["cli", "config", "local_storage", "web_scraper", "weather"];

    // Get global log level from env or use default
    let level = std::env::var("RUST_LOG").unwrap_or_else(|_| "warn".to_string());

    // Build filter string applying the same level to all modules
    let filters = MEMBERS
        .iter()
        .map(|m| format!("{m}={level}"))
        .collect::<Vec<_>>()
        .join(",");

    let env = Env::default()
        .filter_or("DAILY_BUGLE_LOG", &filters)
        .write_style_or("DAILY_BUGLE_LOG_STYLE", "always");

    env_logger::Builder::from_env(env).init();

    trace!("Logging initialized with level: {level}");
}

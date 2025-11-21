use env_logger::Env;
use log::warn;

pub fn init_logging() {
    const MEMBERS: [&str; 6] = [
        "cli",
        "config",
        "local_storage",
        "web_scraper",
        "career",
        "third_party_api",
    ];

    // Get global log level from env or use default
    let level = std::env::var("RUST_LOG").unwrap_or_else(|_| "info".to_string());

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

    warn!("Logging initialized with level: {level}");
}

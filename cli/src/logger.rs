use env_logger::Env;
use log::trace;

pub fn init_logging() {
    const MEMBERS: [&str; 5] = ["cli", "config", "local_storage", "web_scraper", "weather"];
    let filters = MEMBERS.join("=trace,");
    let env = Env::default()
        .filter_or("DAILY_BUGLE_LOG", format!("{}=debug", filters))
        .write_style_or("DAILY_BUGLE_LOG_STYLE", "always");

    env_logger::Builder::from_env(env).init();

    trace!("Logging Initialized");
}

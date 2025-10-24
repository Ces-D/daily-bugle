use clap::ValueEnum;

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum TimeOutTimePeriod {
    Today,
    Week,
    Weekend,
    Month,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum TechnicalArticleSource {
    Aws,
    Figma,
    Google,
    HackernewsNews,
    HackernewsJobs,
    ArminRonacher,
    Mdn,
    Notion,
    Openai,
    Uber,
}

#[derive(Debug, clap::Subcommand)]
pub enum Command {
    #[clap(about = "See the weather in your area")]
    Weather { postal_code: String, complete: bool },
    #[clap(about = "Get a random article from the list of technical article sources")]
    TechnicalArticle {
        #[clap(short, long, help = "Filter the list of sources")]
        sources: Vec<TechnicalArticleSource>,
    },
    #[clap(about = "Get the list of things to do in nyc")]
    NYCEvent { period: TimeOutTimePeriod },
    #[clap(about = "Set or get countdowns ")]
    TestNode,
}

#[derive(Debug, clap::Parser)]
#[clap(author, version, bin_name = "daily-bugle", subcommand_required = true)]
pub struct App {
    #[clap(subcommand)]
    pub command: Command,
}

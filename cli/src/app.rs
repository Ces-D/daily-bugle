use crate::{reminder_command::ReminderArgs, social_command::SocialArgs, tech_command::TechArgs};

#[derive(Debug, clap::Subcommand)]
pub enum Command {
    #[clap(about = "See the weather in your area")]
    Weather {
        postal_code: String,
        complete: bool,
    },

    #[clap(about = "Get a random article from the list of technical article sources")]
    Technical(TechArgs),

    Social(SocialArgs),

    Reminder(ReminderArgs),
    #[clap(about = "Set or get countdowns ")]
    TestNode,
}

#[derive(Debug, clap::Parser)]
#[clap(author, version, bin_name = "daily-bugle", subcommand_required = true)]
pub struct App {
    #[clap(subcommand)]
    pub command: Command,
}

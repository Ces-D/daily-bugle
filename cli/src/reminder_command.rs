use clap::{Parser, Subcommand};

#[derive(Debug, Subcommand)]
pub enum ReminderCommand {
    // TODO: We need CRUD for reminders, and clearing expired
    #[clap(about = "View stored reminders and important dates")]
    List,
}

#[derive(Debug, Parser)]
pub struct ReminderArgs {
    #[clap(subcommand)]
    pub command: ReminderCommand,
}

pub async fn handle_reminder_command(args: ReminderArgs) -> anyhow::Result<()> {
    match args.command {
        ReminderCommand::List => {
            let reminders = reminders::read_reminders_file().await?;
            serde_json::to_writer_pretty(&std::io::stdout(), &reminders)?;
            Ok(())
        }
    }
}

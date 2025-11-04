use clap::{Parser, Subcommand};

#[derive(Debug, Subcommand)]
pub enum ReminderCommand {
    Create {
        prompt: String,
    },
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
        ReminderCommand::Create { prompt } => todo!(),
        ReminderCommand::List => {
            let reminders = reminders::read_reminders_file()?;
            serde_json::to_writer_pretty(&std::io::stdout(), &reminders)?;
            Ok(())
        }
    }
}

use anyhow::Ok;
use clap::{Parser, Subcommand};
use log::info;

#[derive(Debug, Subcommand)]
pub enum TechCommand {
    #[clap(about = "Generate a git commit message")]
    GitCommit {
        #[clap(long, short, default_value = "gpt-5.1-2025-11-13")]
        model: Option<String>,
    },
    #[clap(about = "Generate a pull request message")]
    PullRequest {
        #[clap(long, short, default_value = "gpt-5.1-2025-11-13")]
        model: Option<String>,
    },
}

#[derive(Debug, Parser)]
pub struct TechArgs {
    #[clap(subcommand)]
    pub command: TechCommand,
}

pub async fn handle_tech_command(args: TechArgs) -> anyhow::Result<()> {
    match args.command {
        TechCommand::GitCommit { model } => {
            let commit_message =
                git::git_commit_message(model.expect("We provided a default model").as_str())
                    .await?;
            info!("Commit message Generated Succesfully");
            println!("{}", commit_message);
            Ok(())
        }
        TechCommand::PullRequest { model } => {
            let pr_message =
                git::git_pull_request_message(model.expect("We provided a default model").as_str())
                    .await?;
            info!("Pull Request message Generated Succesfully");
            println!("{}", pr_message);
            Ok(())
        }
    }
}

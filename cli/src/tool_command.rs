use clap::{Parser, Subcommand};

#[derive(Debug, Subcommand)]
pub enum ToolCommand {
    #[clap(about = "Generate a cover letter")]
    CoverLetter {
        #[clap(long, short, default_value = "gpt-5.1-2025-11-13")]
        model: Option<String>,
    },
    #[clap(about = "Generate a professional summary")]
    ProfessionalSummary {
        #[clap(long, short, default_value = "gpt-5.1-2025-11-13")]
        model: Option<String>,
    },
}

#[derive(Debug, Parser)]
pub struct ToolArgs {
    #[clap(subcommand)]
    pub command: ToolCommand,
}

pub async fn handle_tool_command(
    args: ToolArgs,
    career: config::configuration::Career,
) -> anyhow::Result<()> {
    match args.command {
        ToolCommand::CoverLetter { model } => {
            let cover_letter = career::generate_cover_letter(
                &career.resume.path,
                career.resume.headings.clone(),
                &career.job,
                &career.profile,
                &model.expect("We provided a default model"),
            )
            .await?;
            println!("{}", cover_letter);
            Ok(())
        }
        ToolCommand::ProfessionalSummary { model } => {
            let summary = career::generate_professional_summary(
                &career.resume.path,
                career.resume.headings,
                &career.job,
                &career.profile,
                &model.expect("We provided a default model"),
            )
            .await?;
            println!("{}", summary);
            Ok(())
        }
    }
}

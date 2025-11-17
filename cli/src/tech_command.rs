use clap::{Parser, Subcommand, ValueEnum};
use log::info;
use web_scraper::ScrapedEngineeringItems;

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
    ImpervaApplicationSecurity,
    ImpervaAvailability,
    ImpervaDataSecurity,
    ImpervaDdos,
    ImpervaPerformance,
}

#[derive(Debug, Subcommand)]
pub enum TechCommand {
    #[clap(about = "Get a random article from a list of sources")]
    RandomArticle {
        sources: Vec<TechnicalArticleSource>,
    },
    #[clap(about = "Generate a git commit message")]
    GitCommit {
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
        TechCommand::RandomArticle { sources } => {
            let mut entries: ScrapedEngineeringItems = vec![];
            for source in sources {
                match source {
                    TechnicalArticleSource::Aws => {
                        let aws = web_scraper::aws::scrape_aws_engineering_sitemap().await?;
                        entries.extend(aws);
                    }
                    TechnicalArticleSource::Figma => {
                        let figma = web_scraper::figma::scrape_figma_engineering_blog().await?;
                        entries.extend(figma);
                    }
                    TechnicalArticleSource::Google => {
                        let google =
                            web_scraper::google::scrape_google_developer_blogs_sitemap().await?;
                        entries.extend(google);
                    }
                    TechnicalArticleSource::HackernewsNews => {
                        let hackernews_news =
                            web_scraper::hackernews::scrape_hackernews_news(None).await?;
                        entries.extend(hackernews_news);
                    }
                    TechnicalArticleSource::HackernewsJobs => {
                        let hackernews_jobs =
                            web_scraper::hackernews::scrape_hackernews_jobs(None).await?;
                        entries.extend(hackernews_jobs);
                    }
                    TechnicalArticleSource::ArminRonacher => {
                        let lucumr = web_scraper::lucumr::scrape_lucumr_atom_feed().await?;
                        entries.extend(lucumr);
                    }
                    TechnicalArticleSource::Mdn => {
                        let mdn = web_scraper::mdn::scrape_mdn_sitemap().await?;
                        entries.extend(mdn);
                    }
                    TechnicalArticleSource::Notion => {
                        let notion = web_scraper::notion::scrape_notion_blog_sitemap().await?;
                        entries.extend(notion);
                    }
                    TechnicalArticleSource::Openai => {
                        let openai = web_scraper::openai::scrape_openai_sitemap().await?;
                        entries.extend(openai);
                    }
                    TechnicalArticleSource::Uber => {
                        let uber = web_scraper::uber::scrape_uber_engineering_blog().await?;
                        entries.extend(uber);
                    }
                    TechnicalArticleSource::ImpervaApplicationSecurity => {
                        let imperva_application_security =
                            web_scraper::imperva::scrape_imperva_application_security_sitemap()
                                .await?;
                        entries.extend(imperva_application_security);
                    }
                    TechnicalArticleSource::ImpervaAvailability => {
                        let imperva_availability =
                            web_scraper::imperva::scrape_imperva_availability_sitemap().await?;
                        entries.extend(imperva_availability);
                    }
                    TechnicalArticleSource::ImpervaDataSecurity => {
                        let imperva_data_security =
                            web_scraper::imperva::scrape_imperva_data_security_sitemap().await?;
                        entries.extend(imperva_data_security);
                    }
                    TechnicalArticleSource::ImpervaDdos => {
                        let imperva_ddos =
                            web_scraper::imperva::scrape_imperva_ddos_sitemap().await?;
                        entries.extend(imperva_ddos);
                    }
                    TechnicalArticleSource::ImpervaPerformance => {
                        let imperva_performance =
                            web_scraper::imperva::scrape_imperva_performance_sitemap().await?;
                        entries.extend(imperva_performance);
                    }
                }
            }
            let random_index = rand::random_range(..entries.len());
            let random_entry = entries.get(random_index).unwrap();
            serde_json::to_writer_pretty(&std::io::stdout(), &random_entry)?;
            Ok(())
        }
        TechCommand::GitCommit { model } => {
            let commit_message =
                git::git_commit_message(model.expect("We provided a default model").as_str())
                    .await?;
            info!("Commit message Generated Succesfully");
            println!("{}", commit_message);
            Ok(())
        }
    }
}

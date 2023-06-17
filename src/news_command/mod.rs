use clap::{Arg, ArgAction, ArgMatches, Command};

pub struct NewsCommandFactory {}

impl NewsCommandFactory {
    pub fn new() -> NewsCommandFactory {
        NewsCommandFactory {}
    }

    pub fn generate(&self) -> Command {
        Command::new("news")
            .about("Get the latest news")
            .arg(
                Arg::new("topic")
                    .short('t')
                    .help("Topic articles related to")
                    .action(ArgAction::Set)
                    .default_value("general"),
            )
            .arg(
                Arg::new("limit")
                    .short('l')
                    .help("Limit the number of articles")
                    .action(ArgAction::Set)
                    .value_parser(clap::value_parser!(u8))
                    .default_value("10"),
            )
    }

    pub fn handle(self, matches: &ArgMatches) {
        let topic = matches.get_one::<String>("topic").unwrap();
        let limit = matches.get_one::<u8>("limit").unwrap();
        // TODO: make the request to the news apis using these value_parser
        // TODO: output the results of the news api
        println!("Getting {} articles about {}.", limit, topic);
    }
}

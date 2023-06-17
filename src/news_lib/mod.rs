// TODO: this should make sure that the correct configs are provided in the config file

// TODO: integrate into this - https://free-docs.newscatcherapi.com/#introduction
// TODO: integrate into this - https://developer.nytimes.com/docs/most-popular-product/1/overview
// TODO: integrate into this - https://open-platform.theguardian.com/documentationl

struct NewsLib {
    config: Config,
}

impl NewsLib {
    pub fn new(config: Config) -> NewsLib {
        NewsLib { config }
    }
}

// ~~~~~~~~~~~~~~~~~~~~ Culture ~~~~~~~~~~~~~~~~~~~~
// TIMEOUT NYC
pub const TODAY_EVENTS_URL: &str =
    "https://www.timeout.com/newyork/things-to-do/things-to-do-in-new-york-today";
pub const WEEK_EVENTS_URL: &str =
    "https://www.timeout.com/newyork/things-to-do/things-to-do-in-new-york-this-week";
pub const WEEKEND_EVENTS_URL: &str =
    "https://www.timeout.com/newyork/things-to-do/things-to-do-in-nyc-this-weekend";
pub const JANUARY_EVENTS_URL: &str =
    "https://www.timeout.com/newyork/events-calendar/january-events-calendar";
pub const FEBRUARY_EVENTS_URL: &str =
    "https://www.timeout.com/newyork/events-calendar/february-events-calendar";
pub const MARCH_EVENTS_URL: &str =
    "https://www.timeout.com/newyork/events-calendar/march-events-calendar";
pub const APRIL_EVENTS_URL: &str =
    "https://www.timeout.com/newyork/events-calendar/april-events-calendar";
pub const MAY_EVENTS_URL: &str =
    "https://www.timeout.com/newyork/events-calendar/may-events-calendar";
pub const JUNE_EVENTS_URL: &str =
    "https://www.timeout.com/newyork/events-calendar/june-events-calendar";
pub const JULY_EVENTS_URL: &str =
    "https://www.timeout.com/newyork/events-calendar/july-events-calendar";
pub const AUGUST_EVENTS_URL: &str =
    "https://www.timeout.com/newyork/events-calendar/august-events-calendar";
pub const SEPTEMBER_EVENTS_URL: &str =
    "https://www.timeout.com/newyork/events-calendar/september-events-calendar";
pub const OCTOBER_EVENTS_URL: &str =
    "https://www.timeout.com/newyork/events-calendar/october-events-calendar";
pub const NOVEMBER_EVENTS_URL: &str =
    "https://www.timeout.com/newyork/events-calendar/november-events-calendar";
pub const DECEMBER_EVENTS_URL: &str =
    "https://www.timeout.com/newyork/events-calendar/december-events-calendar";

pub const TIMEOUT_STORAGE_PREFIX: &str = "time-out-nyc";

// ~~~~~~~~~~~~~~~~~~~~ Tech ~~~~~~~~~~~~~~~~~~~~
// ARMIN RONACHER
pub const ARMIN_RONACHER_ATOM_FEED_URL: &str = "https://lucumr.pocoo.org/feed.atom";

// HACKER NEWS
pub const HACKER_NEWS_NEWS_URL: &str = "https://news.ycombinator.com/news"; //  Pagination via `?p={}`
pub const HACKER_NEWS_JOBS_URL: &str = "https://news.ycombinator.com/jobs";

// MDN
pub const MDN_SITEMAP_URL: &str = "https://developer.mozilla.org/sitemaps/en-us/sitemap.xml.gz";

// OpenAI
pub const OPENAI_SITEMAP_URL: &str = "https://openai.com/sitemap.xml/page/";

// GOOGLE Developer Blogs
pub const GOOGLE_DEVELOPER_BLOGS_SITEMAP_URL: &str =
    "https://developers.googleblog.com/sitemap.xml";

// STRIPE Blog Sitemap
// PSA it seems like the sitemaps are split into partitions eventually expect partition-6
pub const STRIPE_ENGINEERING_SITEMAP_URL: &str = "https://stripe.com/sitemap/partition-5.xml";

// NOTION
pub const NOTION_TECH_BLOG_URL: &str = "https://www.notion.com/blog/topic/tech";
pub const NOTION_BLOG_SITEMAP_URL: &str = "https://www.notion.com/blog/sitemap.xml";

// FIGMA Engineering Blog
pub const FIGMA_ENGINEERING_BLOG_URL: &str = "https://www.figma.com/blog/engineering"; // Pagination via `?page={}`

// UBER Engineering Blog
pub const UBER_ENGINEERING_BLOG_URL: &str = "https://www.uber.com/blog/new-york/engineering/page/"; // Pagination just add 1 || 2

pub const AWS_ENGINEERING_BLOG_SITEMAP_URL: &str = "https://aws.amazon.com/sitemaps/sitemap_blogs/";

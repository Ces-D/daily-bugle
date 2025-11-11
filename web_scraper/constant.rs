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
pub const ARMIN_RONACHER_STORAGE_CONSTANT: &str = "armin_ronacher";

// HACKER NEWS
pub const HACKER_NEWS_NEWS_URL: &str = "https://news.ycombinator.com/news"; //  Pagination via `?p={}`
pub const HACKER_NEWS_NEWS_STORAGE_CONSTANT: &str = "hacker_news_news";
pub const HACKER_NEWS_JOBS_URL: &str = "https://news.ycombinator.com/jobs";
pub const HACKER_NEWS_JOBS_STORAGE_CONSTANT: &str = "hacker_news_jobs";

// MDN
pub const MDN_SITEMAP_URL: &str = "https://developer.mozilla.org/sitemaps/en-us/sitemap.xml.gz";
pub const MDN_SITEMAP_STORAGE_CONSTANT: &str = "mdn_sitemap";

// OpenAI
pub const OPENAI_SITEMAP_URL: &str = "https://openai.com/sitemap.xml/page/";
pub const OPENAI_SITEMAP_STORAGE_CONSTANT: &str = "openai_sitemap";

// GOOGLE Developer Blogs
pub const GOOGLE_DEVELOPER_BLOGS_SITEMAP_URL: &str =
    "https://developers.googleblog.com/sitemap.xml";
pub const GOOGLE_DEVELOPER_BLOGS_STORAGE_CONSTANT: &str = "google_developer_blogs";

// NOTION
// pub const NOTION_TECH_BLOG_URL: &str = "https://www.notion.com/blog/topic/tech";
pub const NOTION_BLOG_SITEMAP_URL: &str = "https://www.notion.com/blog/sitemap.xml";
pub const NOTION_BLOG_SITEMAP_STORAGE_CONSTANT: &str = "notion_blog";

// FIGMA Engineering Blog
pub const FIGMA_ENGINEERING_BLOG_URL: &str = "https://www.figma.com/blog/engineering";
pub const FIGMA_ENGINEERING_BLOG_STORAGE_CONSTANT: &str = "figma_blog";

// UBER Engineering Blog
pub const UBER_ROOT_URL: &str = "https://www.uber.com";
pub const UBER_ENGINEERING_BLOG_URL: &str = "https://www.uber.com/blog/new-york/engineering/page/"; // Pagination just add 1 || 2
pub const UBER_ENGINEERING_BLOG_STORAGE_CONSTANT: &str = "uber_blog";

// AWS Engineering Blog
pub const AWS_ENGINEERING_BLOG_SITEMAP_URL: &str = "https://aws.amazon.com/sitemaps/sitemap_blogs/";
pub const AWS_ENGINEERING_BLOG_STORAGE_CONSTANT: &str = "aws_blog";

// Imperva Learn Engineering
pub const IMPERVA_LEARN_APPLICATION_SECURITY_SITEMAP_URL: &str =
    "https://www.imperva.com/learn/application_security-sitemap.xml";
pub const IMPERVA_LEARN_DATA_SECURITY_SITEMAP_URL: &str =
    "https://www.imperva.com/learn/data_security-sitemap.xml";
pub const IMPERVA_LEARN_DDOS_SITEMAP_URL: &str = "https://www.imperva.com/learn/ddos-sitemap.xml";
pub const IMPERVA_LEARN_AVAILABILITY_SITEMAP_URL: &str =
    "https://www.imperva.com/learn/availability-sitemap.xml";
pub const IMPERVA_LEARN_PERFORMANCE_SITEMAP_URL: &str =
    "https://www.imperva.com/learn/performance-sitemap.xml";

// Netflix Tech Blog
pub const NETFLIX_TECH_BLOG_SITEMAP_URL: &str = "https://netflixtechblog.com/sitemap/sitemap.xml";

// Github Blog
pub const GITHUB_BLOG_SITEMAP_URL: &str = "https://github.blog/post-sitemap4.xml";

// Medium Engineering Blog
pub const MEDIUM_ENGINEERING_BLOG_SITEMAP_URL: &str =
    "https://medium.engineering/sitemap/sitemap.xml";

// NyTimes Open Blog
pub const NYTIMES_OPEN_BLOG_SITEMAP_URL: &str = "https://open.nytimes.com/sitemap/sitemap.xml";

// Stripe Engineering Blog
pub const STRIPE_ENGINEERING_BLOG_SITEMAP_URL: &str = "https://stripe.com/sitemap/partition-5.xml";

// Square Engineering Blog
pub const SQUARE_ENGINEERING_BLOG_SITEMAP_URL: &str =
    "https://developer.squareup.com/blog/sitemap-0.xml";

// Dan Abramov Blog
pub const DAN_ABORMOV_BLOG_URL: &str = "https://overreacted.io/";

// Deep Learning - Andrew Ng Blog
pub const DEEP_LEARNING_SITEMAP_URL: &str = "https://www.deeplearning.ai/sitemap-1.xml";

// Etsy Code as Craft
pub const ETSY_CODE_AS_CRAFT_URL: &str = "https://www.etsy.com/codeascraft";

// Lea Verou Blog
pub const LEA_VEROU_BLOG_URL: &str = "https://lea.verou.me/blog/"; // Get years posts by adding /2025

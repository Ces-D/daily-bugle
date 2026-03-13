use crate::{
    ScrapedEngineeringItems,
    constant::{GITHUB_BLOG_SITEMAP_URL, GITHUB_BLOG_STORAGE_CONSTANT},
    xml::{CommonXMLHandler, parse_xml_with, request_url_document_text},
};
use anyhow::Result;
use local_storage::key::StorageKey;
use quick_xml::Reader;

pub async fn scrape_github_blog_sitemap() -> Result<ScrapedEngineeringItems> {
    match local_storage::find_stored_item(GITHUB_BLOG_STORAGE_CONSTANT).await {
        Some(i) => Ok(i),
        None => {
            let res = request_url_document_text(GITHUB_BLOG_SITEMAP_URL, None).await?;
            let reader = Reader::from_str(&res);
            let items = parse_xml_with(reader, CommonXMLHandler::default())?;
            let storage_key = StorageKey::new(GITHUB_BLOG_STORAGE_CONSTANT, None, Some(10 * 24));
            local_storage::write_item_to_storage(storage_key, &items).await;
            Ok(items)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{DateTime, Utc};

    #[test]
    fn test_parse_xml_urls() {
        let xml = r#"
<url>
		<loc>https://github.blog/ai-and-ml/github-copilot/preview-referencing-public-code-in-github-copilot/</loc>
		<lastmod>2022-11-01T19:00:20+00:00</lastmod>
		<image:image>
			<image:loc>https://github.blog/wp-content/uploads/2022/06/Copilot.jpeg</image:loc>
		</image:image>
		<image:image>
			<image:loc>https://github.blog/wp-content/uploads/2022/11/copilotpreview1.png?w=1024</image:loc>
		</image:image>
	</url>
	<url>
		<loc>https://github.blog/open-source/gaming/game-off-2022-theme-announcement/</loc>
		<lastmod>2022-11-01T21:13:17+00:00</lastmod>
		<image:image>
			<image:loc>https://github.blog/wp-content/uploads/2022/10/game-off-social-cards.png</image:loc>
		</image:image>
		<image:image>
			<image:loc>https://github.blog/wp-content/uploads/2022/11/game-on.gif</image:loc>
		</image:image>
	</url>
	<url>
		<loc>https://github.blog/news-insights/company-news/github-availability-report-october-2022/</loc>
		<lastmod>2022-11-02T16:00:33+00:00</lastmod>
		<image:image>
			<image:loc>https://github.blog/wp-content/uploads/2022/03/Engineering@2x.png</image:loc>
		</image:image>
	</url>
	<url>
		<loc>https://github.blog/news-insights/company-news/all-in-for-students-expanding-the-next-generation-of-open-source-leaders/</loc>
		<lastmod>2022-11-02T13:42:49+00:00</lastmod>
		<image:image>
			<image:loc>https://github.blog/wp-content/uploads/2022/02/Community-Company_teal-orange@2x.png</image:loc>
		</image:image>
	</url>
	<url>
		<loc>https://github.blog/news-insights/product-news/github-partners-with-arm-to-revolutionize-internet-of-things-software-development-with-github-actions/</loc>
		<lastmod>2022-11-02T18:39:04+00:00</lastmod>
		<image:image>
			<image:loc>https://github.blog/wp-content/uploads/2021/11/GitHub-Actions_social.png</image:loc>
		</image:image>
	</url>
	<url>
		<loc>https://github.blog/news-insights/policy-news-and-insights/advocating-for-developers-to-the-us-copyright-office/</loc>
		<lastmod>2022-11-03T16:00:28+00:00</lastmod>
		<image:image>
			<image:loc>https://github.blog/wp-content/uploads/2022/06/Policy-Open-Source@2x.png</image:loc>
		</image:image>
	</url>"#;
        let reader = Reader::from_str(xml);
        let entries = parse_xml_with(reader, CommonXMLHandler::default())
            .expect("Failed to parse xml content");
        assert_eq!(entries.len(), 6);
        let first = entries.first().unwrap();
        assert_eq!(
            first.url,
            "https://github.blog/ai-and-ml/github-copilot/preview-referencing-public-code-in-github-copilot/"
        );
        assert!(first.updated.is_some());
        let last = entries.last().unwrap();
        assert_eq!(
            last.url,
            "https://github.blog/news-insights/policy-news-and-insights/advocating-for-developers-to-the-us-copyright-office/"
        );
        assert_eq!(
            last.updated,
            Some(
                "2022-11-03T16:00:28+00:00"
                    .parse::<DateTime<Utc>>()
                    .unwrap()
            )
        )
    }
}

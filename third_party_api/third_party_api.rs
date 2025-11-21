pub mod news;
pub mod weather;

trait IntoUrl {
    fn into_url(self) -> url::Url;
}

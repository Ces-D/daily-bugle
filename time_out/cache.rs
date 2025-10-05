use crate::{
    constant::TIMEOUT_STORAGE_PREFIX,
    scrape::{ThingsToDo, ThingsToDoCycle},
};
use chrono::DateTime;
use chrono_tz::Tz;
use log::{error, trace};
use std::path::PathBuf;

pub fn find_cached_item(
    variant: ThingsToDoCycle,
    article_time: DateTime<Tz>,
    cache_path: &PathBuf,
) -> Option<ThingsToDo> {
    let cache_key = fs_cache::FsStore::make_key(
        &format!("{}-{}", TIMEOUT_STORAGE_PREFIX, &variant.to_string()),
        &article_time.to_rfc3339(),
    );
    if let Some(store) = fs_cache::FsStore::new(cache_path).ok() {
        if store.has(&cache_key) {
            if let Some(bytes) = store.get(&cache_key).ok() {
                trace!("Using cached item: {}", cache_key);
                return serde_json::from_slice::<ThingsToDo>(&bytes).ok();
            }
        }
    }
    None
}

pub fn write_item_to_cache(
    variant: ThingsToDoCycle,
    article_time: DateTime<Tz>,
    item: &ThingsToDo,
    cache_path: PathBuf,
) -> Option<String> {
    if let Some(store) = fs_cache::FsStore::new(cache_path).ok() {
        if let Some(i) = serde_json::to_vec(&item).ok() {
            return match store.set_ud(
                &format!("{}-{}", TIMEOUT_STORAGE_PREFIX, &variant.to_string()),
                &article_time.to_rfc3339(),
                i,
            ) {
                Ok(key) => {
                    trace!("Wrote item to cache: {}", key);
                    Some(key)
                }
                Err(e) => {
                    error!("Failed to write item to cache: {:?}", e);
                    None
                }
            };
        }
    }
    None
}

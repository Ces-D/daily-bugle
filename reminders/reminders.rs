use anyhow::{Context, Ok};
use local_storage::key::StorageKey;

mod important_date;

const REMINDERS_FILE: &str = "reminders";

fn storage_key() -> StorageKey {
    StorageKey::new(REMINDERS_FILE, None, Some(365))
}

pub async fn read_reminders_file() -> anyhow::Result<important_date::Reminders> {
    match local_storage::find_stored_item::<important_date::Reminders>(REMINDERS_FILE).await {
        Some(content) => Ok(content),
        None => {
            let default_dates = important_date::defaults()?;
            let key = storage_key();
            local_storage::write_item_to_storage(key, &default_dates)
                .await
                .with_context(|| "Failed to write reminders_file")?;
            Ok(default_dates)
        }
    }
}

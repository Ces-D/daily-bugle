pub mod key;

use crate::key::StorageKey;
use log::{error, info, trace, warn};
use std::{collections::HashSet, path::PathBuf};
use tokio::{
    fs,
    io::{AsyncReadExt, AsyncWriteExt, Error, ErrorKind, Result},
};

struct LocalStorage {
    keys: HashSet<StorageKey>,
    storage_loc: PathBuf,
}

impl LocalStorage {
    pub async fn new_async() -> Result<Self> {
        let loc = config::local_storage_dir_location();
        if loc.exists() && loc.is_dir() {
            let mut keys = HashSet::new();
            let mut entries = fs::read_dir(&loc).await?;
            while let Some(entry) = entries.next_entry().await? {
                let entry_path = entry.path();
                let metadata = entry.metadata().await?;
                if metadata.is_file() {
                    let local_storage_key = StorageKey::from(entry_path.clone());
                    if local_storage_key.is_expired() {
                        fs::remove_file(entry_path.clone()).await?;
                        warn!("Removed expired storage item: {:?}", entry_path);
                        continue;
                    } else {
                        let inserted = keys.insert(local_storage_key);
                        if !inserted {
                            // Failed equivalence so we should remove
                            fs::remove_file(entry_path.clone()).await?;
                            warn!("Removed duplicate storage item: {:?}", entry_path);
                        }
                    }
                }
            }
            Ok(Self {
                keys,
                storage_loc: loc,
            })
        } else {
            fs::create_dir_all(&loc).await?;
            trace!("Created local storage directory at {:?}", loc);
            Ok(Self {
                keys: HashSet::new(),
                storage_loc: loc,
            })
        }
    }

    pub fn contains(&self, key: &StorageKey) -> bool {
        self.keys.contains(key)
    }

    pub async fn insert_item<K: AsRef<[u8]>>(&mut self, key: &StorageKey, item: K) -> Result<()> {
        if self.contains(key) {
            trace!("Not inserting item: {:?}", key);
            Ok(())
        } else {
            let insert_path = self.storage_loc.clone().join(Into::<PathBuf>::into(key));
            if insert_path.exists() {
                error!("Insert path already exists: {:?}", insert_path);
                return Err(Error::new(
                    ErrorKind::AlreadyExists,
                    format!("File  already exists: {:?}", insert_path),
                ));
            } else {
                let mut f = fs::File::create(insert_path).await?;
                f.write_all(item.as_ref()).await?;
                self.keys.insert(key.clone());
                trace!("Inserted item with key {:?}", key);
                Ok(())
            }
        }
    }

    pub async fn get_item(&self, key: &StorageKey) -> Option<Vec<u8>> {
        if self.contains(key) {
            if let Some(cache_key) = self.keys.get(key) {
                // Build full path by joining storage directory with the item key
                let cache_key_path: PathBuf = cache_key.into();
                let item_path = self.storage_loc.clone().join(cache_key_path);
                let mut buf = Vec::new();
                let mut f = fs::File::open(item_path).await.ok()?;
                let out = match f.read_to_end(&mut buf).await {
                    Ok(_) => Some(buf),
                    Err(e) => {
                        warn!("Failed to read cache file: {:?}", e);
                        None
                    }
                };
                out
            } else {
                None
            }
        } else {
            None
        }
    }
}

/// Find a stored item by its constant name. Helper function since implementation is always same.
pub async fn find_stored_item<T: serde::de::DeserializeOwned>(constant: &str) -> Option<T> {
    trace!("Searching for stored item: {}", constant);
    let storage_key = StorageKey::new(&constant, None, None);
    let storage = LocalStorage::new_async().await.ok()?;
    if let Some(bytes) = storage.get_item(&storage_key).await {
        info!("Using cached item: {:?}", storage_key);
        serde_json::from_slice::<T>(&bytes).ok()
    } else {
        None
    }
}

/// Write an item to storage. Helper function since implementation is always same.
pub async fn write_item_to_storage<T: serde::Serialize>(
    storage_key: StorageKey,
    item: &T,
) -> Option<()> {
    trace!("Writing item to storage: {}", storage_key.constant);
    let mut storage = LocalStorage::new_async().await.ok()?;
    match serde_json::to_vec(item) {
        Ok(serialized) => storage.insert_item(&storage_key, serialized).await.ok(),
        Err(e) => {
            error!("Failed to convert item to array: {:?}", e);
            return None;
        }
    }
}

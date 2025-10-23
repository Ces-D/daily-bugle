pub mod key;

use crate::key::StorageKey;
use log::{error, trace, warn};
use std::{
    collections::HashSet,
    io::{Read, Write},
    path::PathBuf,
};

struct LocalStorage {
    keys: HashSet<StorageKey>,
    storage_loc: PathBuf,
}

impl LocalStorage {
    pub fn new() -> std::io::Result<Self> {
        let loc = config::local_storage_dir_location();
        if loc.exists() && loc.is_dir() {
            let mut keys = HashSet::new();
            for entry in std::fs::read_dir(&loc)? {
                let entry_path = entry?.path();
                let local_storage_key = StorageKey::from(entry_path.clone());
                if local_storage_key.is_expired() {
                    std::fs::remove_file(entry_path.clone())?;
                    warn!("Removed expired storage item: {:?}", entry_path);
                    continue;
                } else {
                    let inserted = keys.insert(local_storage_key);
                    if !inserted {
                        // Failed equivalence so we should remove
                        std::fs::remove_file(entry_path.clone())?;
                        warn!("Removed duplicate storage item: {:?}", entry_path);
                    }
                }
            }
            Ok(Self {
                keys,
                storage_loc: loc,
            })
        } else {
            std::fs::create_dir_all(&loc)?;
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

    pub fn insert_item<K: AsRef<[u8]>>(
        &mut self,
        key: &StorageKey,
        item: K,
    ) -> std::io::Result<()> {
        if self.contains(key) {
            Ok(())
        } else {
            let insert_path = self.storage_loc.clone().join(Into::<PathBuf>::into(key));
            if insert_path.exists() {
                error!("Insert path already exists: {:?}", insert_path);
                return Err(std::io::Error::new(
                    std::io::ErrorKind::AlreadyExists,
                    format!("File  already exists: {:?}", insert_path),
                ));
            } else {
                let mut f = std::fs::File::create(insert_path)?;
                f.write_all(item.as_ref())?;
                self.keys.insert(key.clone());
                trace!("Inserted item with key {:?}", key);
                Ok(())
            }
        }
    }

    pub fn get_item(&self, key: &StorageKey) -> std::io::Result<Vec<u8>> {
        let get_path = self.storage_loc.clone().join(Into::<PathBuf>::into(key));
        let mut f = std::fs::File::open(get_path)?;
        let mut buf = Vec::new();
        f.read_to_end(&mut buf)?;
        Ok(buf)
    }
}

/// Find a stored item by its constant name. Helper function since implementation is always same.
pub fn find_stored_item<T: serde::de::DeserializeOwned>(constant: &str) -> Option<T> {
    let storage_key = StorageKey::new(&constant, None, None);
    let storage = LocalStorage::new().ok()?;
    let bytes = storage.get_item(&storage_key).ok()?;
    log::trace!("Using cached item: {:?}", storage_key);
    serde_json::from_slice::<T>(&bytes).ok()
}

/// Write an item to storage. Helper function since implementation is always same.
pub fn write_item_to_storage<T: serde::Serialize>(storage_key: StorageKey, item: &T) -> Option<()> {
    let mut storage = LocalStorage::new().ok()?;
    match serde_json::to_vec(item) {
        Ok(serialized) => storage.insert_item(&storage_key, serialized).ok(),
        Err(e) => {
            log::error!("Failed to convert item to array: {:?}", e);
            return None;
        }
    }
}

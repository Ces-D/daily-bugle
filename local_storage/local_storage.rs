pub mod key;

use crate::key::LocalStorageKey;
use log::{error, trace, warn};
use std::{
    collections::HashSet,
    io::{Read, Write},
    path::PathBuf,
};

pub struct LocalStorage {
    keys: HashSet<LocalStorageKey>,
    storage_loc: PathBuf,
}

impl LocalStorage {
    pub fn new() -> std::io::Result<Self> {
        let loc = config::local_storage_dir_location();
        if loc.exists() && loc.is_dir() {
            let mut keys = HashSet::new();
            for entry in std::fs::read_dir(&loc)? {
                let entry_path = entry?.path();
                let local_storage_key = LocalStorageKey::from(entry_path.clone());
                let inserted = keys.insert(local_storage_key);
                if !inserted {
                    // Failed equivalence so we should remove
                    std::fs::remove_file(entry_path.clone())?;
                    warn!("Removed duplicate storage item: {:?}", entry_path);
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

    pub fn contains(&self, key: &LocalStorageKey) -> bool {
        self.keys.contains(key)
    }

    pub fn insert_item<K: AsRef<[u8]>>(
        &mut self,
        key: &LocalStorageKey,
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

    pub fn get_item(&self, key: &LocalStorageKey) -> std::io::Result<Vec<u8>> {
        let get_path = self.storage_loc.clone().join(Into::<PathBuf>::into(key));
        let mut f = std::fs::File::open(get_path)?;
        let mut buf = Vec::new();
        f.read_to_end(&mut buf)?;
        Ok(buf)
    }
}

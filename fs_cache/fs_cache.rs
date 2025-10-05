use std::fs;
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};

/// A simple local filesystem storage:
/// - Each value is stored in its own file
/// - The filename itself is the key
/// - Keys are built from (user, datetime) for now
///
/// No external crates required.
pub struct FsStore {
    root: PathBuf,
}

impl FsStore {
    /// Create (and ensure) the root directory exists.
    pub fn new<P: AsRef<Path>>(root: P) -> io::Result<Self> {
        let root = root.as_ref().to_path_buf();
        fs::create_dir_all(&root)?;
        Ok(Self { root })
    }

    /// Build a filename-safe key from (user, datetime).
    ///
    /// Rules:
    /// - Only allow [A-Za-z0-9._-] and replace everything else with '_'
    /// - Join with a double underscore to avoid ambiguity
    /// - Add a `.dat` extension for clarity (still part of the key)
    pub fn make_key(user: &str, datetime: &str) -> String {
        fn sanitize(s: &str) -> String {
            s.chars()
                .map(|c| match c {
                    'A'..='Z' | 'a'..='z' | '0'..='9' | '.' | '_' | '-' => c,
                    _ => '_',
                })
                .collect()
        }

        let u = sanitize(user);
        let d = sanitize(datetime);
        format!("{u}__{d}.dat")
    }

    /// Return the absolute path for a given key (filename).
    fn path_for(&self, key: &str) -> PathBuf {
        self.root.join(key)
    }

    /// Check if the key exists.
    pub fn has(&self, key: &str) -> bool {
        self.path_for(key).is_file()
    }

    /// Store bytes at `key`. Fails if the key already exists (enforcing uniqueness).
    ///
    /// If you prefer "upsert" semantics, swap the existence check for a simple write.
    pub fn set<K: AsRef<[u8]>>(&self, key: &str, bytes: K) -> io::Result<()> {
        let p = self.path_for(key);
        if p.exists() {
            return Err(io::Error::new(
                io::ErrorKind::AlreadyExists,
                format!("key already exists: {key}"),
            ));
        }

        // Write atomically-ish: write to a temp then rename.
        let tmp = p.with_extension("tmp");
        {
            let mut f = fs::File::create(&tmp)?;
            f.write_all(bytes.as_ref())?;
            f.sync_all()?;
        }
        fs::rename(tmp, p)
    }

    /// Convenience: set using (user, datetime).
    pub fn set_ud<K: AsRef<[u8]>>(
        &self,
        user: &str,
        datetime: &str,
        bytes: K,
    ) -> io::Result<String> {
        let key = Self::make_key(user, datetime);
        self.set(&key, bytes)?;
        Ok(key)
    }

    /// Read all bytes stored under `key`.
    pub fn get(&self, key: &str) -> io::Result<Vec<u8>> {
        let p = self.path_for(key);
        let mut f = fs::File::open(p)?;
        let mut buf = Vec::new();
        f.read_to_end(&mut buf)?;
        Ok(buf)
    }

    /// Delete a `key`. Succeeds if the file existed and was removed.
    pub fn delete(&self, key: &str) -> io::Result<()> {
        let p = self.path_for(key);
        fs::remove_file(p)
    }

    /// Remove **all** stored entries in the root directory (files only).
    pub fn clear(&self) -> io::Result<()> {
        for entry in fs::read_dir(&self.root)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_file() {
                let _ = fs::remove_file(path);
            }
        }
        Ok(())
    }

    /// List all keys (filenames) currently stored.
    pub fn keys(&self) -> io::Result<Vec<String>> {
        let mut out = Vec::new();
        for entry in fs::read_dir(&self.root)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_file() {
                if let Some(name) = path.file_name().and_then(|s| s.to_str()) {
                    out.push(name.to_string());
                }
            }
        }
        out.sort();
        Ok(out)
    }
}

// ---- Example usage (put in tests or main) ----
#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn it_works() {
        let tmpdir = tempfile_dir();
        let store = FsStore::new(&tmpdir).unwrap();

        let user = "ces@example.com";
        let dt = "2025-10-03T17:05:00-04:00";
        let key = FsStore::make_key(user, dt);

        assert!(!store.has(&key));
        store.set(&key, b"hello world").unwrap();
        assert!(store.has(&key));

        let val = store.get(&key).unwrap();
        assert_eq!(val, b"hello world");

        let keys = store.keys().unwrap();
        assert_eq!(keys, vec![key.clone()]);

        store.delete(&key).unwrap();
        assert!(!store.has(&key));

        // set via user/datetime helper
        let key2 = store.set_ud(user, dt, b"bytes").unwrap();
        assert!(store.has(&key2));

        store.clear().unwrap();
        assert!(store.keys().unwrap().is_empty());
    }

    /// Minimal temp dir helper without external crates.
    fn tempfile_dir() -> PathBuf {
        use std::time::{SystemTime, UNIX_EPOCH};
        let mut base = std::env::temp_dir();
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        base.push(format!("fsstore_{nanos}"));
        fs::create_dir_all(&base).unwrap();
        base
    }
}

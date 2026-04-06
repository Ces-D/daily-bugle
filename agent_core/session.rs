use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

fn sessions_dir() -> Result<PathBuf> {
    let storage = config::application_storage(true)?.join("sessions");
    if !storage.exists() {
        std::fs::create_dir_all(storage.clone())?;
    }
    Ok(storage)
}

/// On-disk representation of a session. Serialized as JSON.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionFile {
    pub id: String,
    pub model: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub messages: Vec<genai::chat::ChatMessage>,
}

#[derive(Debug, Clone)]
pub struct Session {
    pub file: SessionFile,
    path: PathBuf,
}

impl Session {
    pub fn new(model: &str) -> Result<Self> {
        let dir = sessions_dir()?;
        let id = uuid::Uuid::new_v4();
        let now = Utc::now();
        let mut session = Self {
            file: SessionFile {
                id: id.to_string(),
                model: model.to_string(),
                created_at: now,
                updated_at: now,
                messages: vec![],
            },
            path: dir.join(format!("{}.json", id)),
        };
        session.save()?;
        Ok(session)
    }

    pub fn load(session_id: &str) -> Result<Self> {
        let dir = sessions_dir()?;
        let path = dir.join(format!("{session_id}.json"));
        let reader = std::io::BufReader::new(std::fs::File::open(&path)?);
        let file: SessionFile = serde_json::from_reader(reader)?;
        Ok(Self { file, path })
    }

    pub fn save(&mut self) -> Result<()> {
        self.file.updated_at = Utc::now();
        let json = serde_json::to_string(&self.file)?;
        std::fs::write(&self.path, json)?;
        Ok(())
    }

    pub fn list() -> Result<Vec<SessionFile>> {
        let dir = sessions_dir()?;
        let mut sessions: Vec<SessionFile> = std::fs::read_dir(&dir)?
            .filter_map(|entry| entry.ok())
            .filter(|entry| entry.path().extension().is_some_and(|ext| ext == "json"))
            .filter_map(|entry| {
                let reader = std::io::BufReader::new(std::fs::File::open(entry.path()).ok()?);
                serde_json::from_reader::<_, SessionFile>(reader).ok()
            })
            .collect();
        sessions.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
        Ok(sessions)
    }

    pub fn delete(session_id: &str) -> Result<()> {
        let dir = sessions_dir()?;
        let path = dir.join(format!("{session_id}.json"));
        std::fs::remove_file(path)?;
        Ok(())
    }

    /// Create a persist callback that saves messages to this session's file on each call.
    pub fn persist_callback(self) -> (PersistFn, std::sync::Arc<std::sync::Mutex<Session>>) {
        let session = std::sync::Arc::new(std::sync::Mutex::new(self));
        let session_clone = session.clone();
        let callback: PersistFn = Box::new(move |messages| {
            let mut s = session_clone.lock().map_err(|e| anyhow::anyhow!("{e}"))?;
            s.file.messages = messages.to_vec();
            s.save()
        });
        (callback, session)
    }
}

pub type PersistFn = Box<dyn Fn(&[genai::chat::ChatMessage]) -> anyhow::Result<()> + Send + Sync>;

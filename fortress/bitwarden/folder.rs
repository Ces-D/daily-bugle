use super::{CoreCommands, bw};
use anyhow::{Context, Result, bail};
use local_storage::key::StorageKey;
use serde::{Deserialize, Serialize};

const BW_FOLDER_STORAGE_KEY: &str = "bw_folder_storage_key";

#[derive(Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct Folder {
    id: Option<String>,
    object: Option<String>,
    name: String,
}

impl Folder {
    pub fn new(name: String) -> Self {
        Self {
            name,
            ..Default::default()
        }
    }

    pub fn set_name(mut self, name: String) -> Self {
        self.name = name;
        self
    }
}

impl CoreCommands for Folder {
    type ListItem = Self;

    async fn create(&self) -> Result<()> {
        if self.id.is_some() {
            bail!(
                "Cannot create a preexisting folder. You can edit the object but cannot create it again."
            );
        }
        let _json = serde_json::to_string(&self)?;
        bw(vec!["create", "folder"], Some(&_json), true)?;
        if let Some(_) = local_storage::invalidate_stored_item(BW_FOLDER_STORAGE_KEY).await {
            log::info!("Invalidated bw folder storage");
        }
        Ok(())
    }

    fn edit(&self) -> Result<()> {
        if self.id.is_none() {
            bail!("Cannot edit a folder without an id. Use create instead.");
        }
        let _json = serde_json::to_string(&self)?;
        bw(
            vec!["edit", "folder", &self.id.clone().unwrap()],
            Some(&_json),
            true,
        )?;
        Ok(())
    }

    async fn list(&self) -> Result<Vec<Self>> {
        match local_storage::find_stored_item(BW_FOLDER_STORAGE_KEY).await {
            Some(i) => Ok(i),
            None => {
                let res = bw(vec!["list", "folders"], None, false)?;
                let items: Vec<Self> = serde_json::from_str(&res)?;
                let storage_key = StorageKey::new(BW_FOLDER_STORAGE_KEY, None, Some(10 * 7));
                if let Some(_) = local_storage::write_item_to_storage(storage_key, &items).await {
                    log::info!("Writing bw folders to cache");
                }
                Ok(items)
            }
        }
    }

    async fn delete(&self) -> Result<()> {
        if self.id.is_none() {
            bail!("Cannot delete a folder without an id. Use create instead.");
        }
        bw(
            vec!["delete", "folder", &self.id.clone().unwrap()],
            None,
            false,
        )?;
        if let Some(_) = local_storage::invalidate_stored_item(BW_FOLDER_STORAGE_KEY).await {
            log::info!("Invalidated bw folder storage");
        }
        Ok(())
    }

    async fn restore(&self) -> Result<()> {
        if self.id.is_none() {
            bail!("Cannot restore a folder without an id. Use create instead.");
        }
        bw(vec!["restore", &self.id.clone().unwrap()], None, false)?;
        if let Some(_) = local_storage::invalidate_stored_item(BW_FOLDER_STORAGE_KEY).await {
            log::info!("Invalidated bw folder storage");
        }
        Ok(())
    }

    fn get(id: String) -> Result<Self> {
        let res = bw(vec!["get", "folder", &id], None, false)?;
        Self::try_from(res)
    }
}

impl TryFrom<String> for Folder {
    type Error = anyhow::Error;

    fn try_from(value: String) -> std::result::Result<Self, Self::Error> {
        serde_json::from_str(&value)
            .with_context(|| "Could not deserialize the value into a Folder")
    }
}

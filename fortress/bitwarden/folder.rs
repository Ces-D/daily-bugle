use super::{CoreCommands, bw};
use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};

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
    fn create(&self) -> Result<()> {
        if self.id.is_some() {
            bail!(
                "Cannot create a preexisting folder. You can edit the object but cannot create it again."
            );
        }
        let _json = serde_json::to_string(&self)?;
        bw(vec!["create", "folder"], Some(&_json), true)?;
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

    fn list(&self) -> Result<Vec<Self>> {
        let res = bw(vec!["list", "folders"], None, false)?;
        let items: Vec<Self> = serde_json::from_str(&res)?;
        Ok(items)
    }

    fn delete(&self) -> Result<()> {
        if self.id.is_none() {
            bail!("Cannot delete a folder without an id. Use create instead.");
        }
        bw(
            vec!["delete", "folder", &self.id.clone().unwrap()],
            None,
            false,
        )?;
        Ok(())
    }

    fn restore(&self) -> Result<()> {
        if self.id.is_none() {
            bail!("Cannot restore a folder without an id. Use create instead.");
        }
        bw(vec!["restore", &self.id.clone().unwrap()], None, false)?;
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

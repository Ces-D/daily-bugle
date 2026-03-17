use super::{CoreCommands, bw};
use anyhow::{Context, Result, bail};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use serde_repr::{Deserialize_repr, Serialize_repr};

#[derive(Serialize, Deserialize, Default, Clone)]
pub struct Uri {
    pub uri: String,
}

#[derive(Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct Login {
    pub uris: Vec<Uri>,
    pub username: Option<String>,
    pub password: String,
    pub totp: Option<String>,
    pub fido2_credentials: Vec<Value>,
    password_revision_date: Option<DateTime<Utc>>,
}
impl Login {
    pub async fn generate(username: String, length: Option<u8>) -> Result<Self> {
        let length = length.unwrap_or(24);
        let res = bw(
            vec![
                "generate",
                "--uppercase",
                "--lowercase",
                "--number",
                "--special",
                "--length",
                &length.to_string(),
            ],
            None,
            false,
        )?;
        Ok(Self {
            username: Some(username),
            password: res,
            ..Default::default()
        })
    }
}

/// The type field for SecureNote is always 0
#[derive(Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct SecureNote {
    #[serde(rename = "type")]
    pub r_type: u8,
}

#[derive(Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct Card {
    pub cardholder_name: String,
    pub brand: String,
    pub number: String,
    pub exp_month: String,
    pub exp_year: String,
    pub code: String,
}

#[derive(Default, Serialize_repr, Deserialize_repr)]
#[repr(u8)]
pub enum ItemType {
    #[default]
    Login = 1,
    SecureNote = 2,
    Card = 3,
    Identity = 4,
}

#[derive(Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct Item {
    pub id: Option<String>,
    pub password_history: Option<Value>,
    pub revision_date: Option<DateTime<Utc>>,
    pub creation_date: Option<DateTime<Utc>>,
    pub object: Option<String>,
    pub deleted_date: Option<DateTime<Utc>>,
    pub archived_date: Option<DateTime<Utc>>,
    pub organization_id: Option<String>,
    pub collection_ids: Option<Vec<String>>,
    pub folder_id: Option<String>,
    #[serde(rename = "type")]
    pub r_type: ItemType,
    pub name: String,
    pub notes: Option<String>,
    pub favorite: bool,
    pub fields: Vec<String>,
    pub login: Option<Login>,
    pub secure_note: Option<SecureNote>,
    pub card: Option<Card>,
    pub identity: Option<String>,
    pub ssh_key: Option<String>,
    pub reprompt: u8,
    pub attachments: Vec<Value>,
}

impl Item {
    pub fn new(name: String, notes: String) -> Self {
        Self {
            name,
            notes: Some(notes),
            ..Default::default()
        }
    }

    pub fn set_name(mut self, name: String) -> Self {
        self.name = name;
        self
    }

    pub fn set_notes(mut self, notes: String) -> Self {
        self.notes = Some(notes);
        self
    }

    pub fn set_favorite(mut self, favorite: bool) -> Self {
        self.favorite = favorite;
        self
    }

    pub fn set_folder_id(mut self, folder_id: String) -> Self {
        self.folder_id = Some(folder_id);
        self
    }

    /// Signals intent to create a Login item. Sets the item type to Login.
    pub fn set_login(mut self, login: Login) -> Self {
        self.r_type = ItemType::Login;
        self.login = Some(login);
        self.secure_note = None;
        self.card = None;
        self.identity = None;
        self
    }

    /// Signals intent to create a SecureNote item. Sets the item type to SecureNote.
    pub fn set_secure_note(mut self, secure_note: SecureNote) -> Self {
        self.r_type = ItemType::SecureNote;
        self.secure_note = Some(secure_note);
        self.login = None;
        self.card = None;
        self.identity = None;
        self
    }

    /// Signals intent to create a Card item. Sets the item type to Card.
    pub fn set_card(mut self, card: Card) -> Self {
        self.r_type = ItemType::Card;
        self.card = Some(card);
        self.login = None;
        self.secure_note = None;
        self.identity = None;
        self
    }
    /// Signals intent to create an Identity item. Sets the item type to Identity.
    pub fn set_identity(mut self, identity: String) -> Self {
        self.r_type = ItemType::Identity;
        self.identity = Some(identity);
        self.login = None;
        self.secure_note = None;
        self.card = None;
        self
    }
}

impl CoreCommands for Item {
    fn create(&self) -> Result<()> {
        if self.id.is_some() {
            bail!(
                "Cannot create a preexisting item. You can edit the object but cannot create it again."
            );
        }
        let _json = serde_json::to_string(&self)?;
        bw(vec!["create", "item"], Some(&_json), true)?;
        Ok(())
    }

    fn edit(&self) -> Result<()> {
        if self.id.is_none() {
            bail!("Cannot edit an item without an id. Use create instead.");
        }
        let _json = serde_json::to_string(&self)?;
        bw(
            vec!["edit", "item", &self.id.clone().unwrap()],
            Some(&_json),
            true,
        )?;
        Ok(())
    }

    fn list(&self) -> Result<Vec<Self>> {
        let command = vec![
            "list",
            "items",
            "--folderid",
            match &self.folder_id {
                Some(id) => id,
                None => "null",
            },
        ];
        let res = bw(command, None, false)?;
        let items: Vec<Self> = serde_json::from_str(&res)?;
        Ok(items)
    }

    fn delete(&self) -> Result<()> {
        if self.id.is_none() {
            bail!("Cannot delete an item without an id. Use create instead.");
        }
        bw(
            vec!["delete", "item", &self.id.clone().unwrap()],
            None,
            false,
        )?;
        Ok(())
    }

    fn restore(&self) -> Result<()> {
        if self.id.is_none() {
            bail!("Cannot restore an item without an id. Use create instead.");
        }
        bw(
            vec!["restore", "item", &self.id.clone().unwrap()],
            None,
            false,
        )?;
        Ok(())
    }

    fn get(id: String) -> Result<Self> {
        let res = bw(vec!["get", "item", &id], None, false)?;
        Self::try_from(res)
    }
}

impl TryFrom<String> for Item {
    type Error = anyhow::Error;

    fn try_from(value: String) -> std::result::Result<Self, Self::Error> {
        serde_json::from_str(&value).with_context(|| "Could not deserialize the value into an Item")
    }
}

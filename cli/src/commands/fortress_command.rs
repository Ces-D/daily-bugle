use anyhow::Result;
use clap::{Parser, Subcommand};
use fortress::bitwarden::{
    CoreCommands,
    folder::Folder,
    item::{Item, Login, SecureNote},
};

#[derive(Debug, Parser)]
pub struct FortressArgs {
    #[clap(subcommand)]
    pub command: FortressCommand,
}

#[derive(Debug, Subcommand)]
pub enum FortressCommand {
    #[clap(about = "Create a new login entry with a generated password")]
    CreateLogin {
        #[clap(short, long, help = "Account login username")]
        username: String,
        #[clap(short, long, help = "Name of the login account")]
        name: String,
        #[clap(short, long, help = "Password length (default: 24)")]
        length: Option<u8>,
        #[clap(long, default_value = "", help = "Additional notes for the login")]
        notes: String,
        #[clap(short, long, help = "Folder ID to store the login in")]
        folder_id: Option<String>,
    },
    #[clap(about = "Create a new secure note")]
    CreateSecureNote {
        #[clap(short, long, help = "Name of the secure note")]
        name: String,
        #[clap(long, default_value = "", help = "Content of the secure note")]
        notes: String,
        #[clap(short, long, help = "Folder ID to store the note in")]
        folder_id: Option<String>,
    },
    #[clap(about = "Create a new folder")]
    CreateFolder {
        #[clap(short, long, help = "Name of the folder")]
        name: String,
    },
    #[clap(about = "List all items, optionally filtered by folder")]
    ListItems {
        #[clap(short, long, help = "Folder ID to filter items by")]
        folder_id: Option<String>,
    },
    #[clap(about = "Edit an existing item by ID")]
    EditItem {
        #[clap(help = "ID of the item to edit")]
        id: String,
        #[clap(short, long, help = "New name for the item")]
        name: Option<String>,
        #[clap(long, help = "New notes for the item")]
        notes: Option<String>,
        #[clap(short, long, help = "Set item as favorite")]
        favorite: Option<bool>,
        #[clap(long, help = "Folder ID to move the item to")]
        folder_id: Option<String>,
        #[clap(long, help = "Set as identity item with this value")]
        identity: Option<String>,
    },
    #[clap(about = "Delete an item by ID")]
    DeleteItem {
        #[clap(help = "ID of the item to delete")]
        id: String,
    },
    #[clap(about = "Restore a deleted item by ID")]
    RestoreItem {
        #[clap(help = "ID of the item to restore")]
        id: String,
    },
    #[clap(about = "Get an item by ID")]
    GetItem {
        #[clap(help = "ID of the item to get")]
        id: String,
    },
    #[clap(about = "Edit a folder by ID")]
    EditFolder {
        #[clap(help = "ID of the folder to edit")]
        id: String,
        #[clap(short, long, help = "New name for the folder")]
        name: String,
    },
    #[clap(about = "Delete a folder by ID")]
    DeleteFolder {
        #[clap(help = "ID of the folder to delete")]
        id: String,
    },
    #[clap(about = "Restore a deleted folder by ID")]
    RestoreFolder {
        #[clap(help = "ID of the folder to restore")]
        id: String,
    },
    #[clap(about = "List all folders")]
    ListFolders,
}

pub async fn handle_fortress_command(args: FortressArgs) -> Result<()> {
    match args.command {
        FortressCommand::CreateLogin {
            username,
            name,
            length,
            notes,
            folder_id,
        } => {
            let login = Login::generate(username, length).await?;
            let mut item = Item::new(name, notes).set_login(login);
            if let Some(id) = folder_id {
                item = item.set_folder_id(id);
            }
            item.create().await?;
        }
        FortressCommand::CreateSecureNote {
            name,
            notes,
            folder_id,
        } => {
            let mut item = Item::new(name, notes).set_secure_note(SecureNote::default());
            if let Some(id) = folder_id {
                item = item.set_folder_id(id);
            }
            item.create().await?;
        }
        FortressCommand::CreateFolder { name } => {
            let folder = Folder::new(name);
            folder.create().await?;
        }
        FortressCommand::ListItems { folder_id } => {
            let mut item = Item::new(String::new(), String::new());
            if let Some(id) = folder_id {
                item = item.set_folder_id(id);
            }
            let items = item.list().await?;
            for i in items {
                println!("{}", serde_json::to_string_pretty(&i)?);
            }
        }
        FortressCommand::EditItem {
            id,
            name,
            notes,
            favorite,
            folder_id,
            identity,
        } => {
            let mut item = Item::get(id)?;
            if let Some(name) = name {
                item = item.set_name(name);
            }
            if let Some(notes) = notes {
                item = item.set_notes(notes);
            }
            if let Some(favorite) = favorite {
                item = item.set_favorite(favorite);
            }
            if let Some(folder_id) = folder_id {
                item = item.set_folder_id(folder_id);
            }
            if let Some(identity) = identity {
                item = item.set_identity(identity);
            }
            item.edit()?;
        }
        FortressCommand::DeleteItem { id } => {
            let item = Item::get(id)?;
            item.delete().await?;
        }
        FortressCommand::RestoreItem { id } => {
            let item = Item::get(id)?;
            item.restore().await?;
        }
        FortressCommand::GetItem { id } => {
            let item = Item::get(id)?;
            println!("{}", serde_json::to_string_pretty(&item)?);
        }
        FortressCommand::EditFolder { id, name } => {
            let folder = Folder::get(id)?.set_name(name);
            folder.edit()?;
        }
        FortressCommand::DeleteFolder { id } => {
            let folder = Folder::get(id)?;
            folder.delete().await?;
        }
        FortressCommand::RestoreFolder { id } => {
            let folder = Folder::get(id)?;
            folder.restore().await?;
        }
        FortressCommand::ListFolders => {
            let folder = Folder::new(String::new());
            let folders = folder.list().await?;
            for f in folders {
                println!("{}", serde_json::to_string_pretty(&f)?);
            }
        }
    }
    Ok(())
}

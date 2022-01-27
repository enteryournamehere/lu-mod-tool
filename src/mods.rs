use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mods {
    pub version: String,
    pub database: PathBuf,
    pub sqlite: PathBuf,
    #[serde(default)]
    pub resource_folder: PathBuf,
    pub priorities: Vec<ModPriority>,
}

impl Default for Mods {
    fn default() -> Mods {
        Mods {
            version: "".to_string(),
            database: PathBuf::from("cdclient.fdb"),
            sqlite: PathBuf::from("CDServer.sqlite"),
            resource_folder: PathBuf::new(),
            priorities: vec![],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModPriority {
    directory: String,
    priority: u32,
}

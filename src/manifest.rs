use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize)]
pub struct Manifest {
    pub name: String,
    pub files: Vec<PathBuf>,
}

impl Default for Manifest {
    fn default() -> Self {
        Self {
            name: "mod-name".to_string(),
            files: vec![PathBuf::from("mod.json")],
        }
    }
}

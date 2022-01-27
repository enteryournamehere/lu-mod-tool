use crate::locale::*;
use crate::lu_mod::Mod;
use crate::mods::*;
use assembly_fdb::mem::Database;
use std::collections::HashMap;
use std::path::PathBuf;

pub struct ModContext<'a> {
    pub configuration: Mods,
    pub root: PathBuf,
    pub database: Database<'a>,
    pub localization: Localization,
    pub ids: HashMap<String, u32>,
    pub mods: Vec<Mod>,
    pub server_sql: Vec<String>,
}

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
    pub localization: Option<Localization>,
    pub ids: HashMap<String, u32>,
    pub mods: HashMap<String, Mod>,
    pub server_sql: Vec<String>,
}
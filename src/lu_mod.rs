#![allow(unused_variables)]

use crate::ModContext;
use assembly_fdb::common::ValueType;
use assembly_fdb::core::Field;
use color_eyre::eyre::{self, eyre, WrapErr};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Mod {
    pub id: String,
    #[serde(rename = "type")]
    pub mod_type: String,
    pub action: String,
    #[serde(rename = "show-defaults")]
    pub show_defaults: Option<bool>,
    #[serde(default)]
    pub components: Vec<String>,
    pub table: Option<String>,
    // items: Option<Vec<JsonValue>>,
    // skills: Option<Vec<JsonValue>>,
    // tasks: Option<Vec<MissionModTask>>,
    // mission_offers: Option<Vec<MissionOffer>>, // json "missions"
    pub locale: HashMap<String, String>,
    pub values: HashMap<String, serde_json::Value>,
    #[serde(skip)]
    pub defaults: HashMap<String, Field>,
    #[serde(skip)]
    pub fields: Vec<Field>,
}

impl Default for Mod {
    fn default() -> Self {
        Self {
            id: "".to_string(),
            mod_type: "".to_string(),
            action: "add".to_string(),
            show_defaults: None,
            components: vec![],
            table: None,
            // items: None,
            // skills: None,
            // tasks: None,
            // mission_offers: None,
            locale: HashMap::new(),
            values: HashMap::new(),
            defaults: HashMap::new(),
            fields: vec![],
        }
    }
}

impl Mod {
    pub fn set_default(&mut self, key: &str, value: Field) {
        if self.values.contains_key(key) {
            return;
        }

        self.defaults.insert(key.to_string(), value);
    }

    pub fn get_values(&self) {}
}

pub fn apply_item_mod(lu_mod: &mut Mod) -> eyre::Result<()> {
    Ok(())
}

pub fn apply_sql_mod(lu_mod: &Mod) -> eyre::Result<()> {
    Ok(())
}

pub fn apply_environmental_mod(lu_mod: &Mod) -> eyre::Result<()> {
    Ok(())
}

pub fn apply_mission_mod(lu_mod: &Mod) -> eyre::Result<()> {
    Ok(())
}

pub fn apply_npc_mod(lu_mod: &Mod) -> eyre::Result<()> {
    Ok(())
}

pub fn apply_object_mod(mod_context: &ModContext, lu_mod: &mut Mod) -> eyre::Result<()> {
    // TODO items and skills and stuff

    add_row_in_table(mod_context, lu_mod, String::from("Objects"))
}

pub fn add_row(mod_context: &ModContext, lu_mod: &mut Mod) -> eyre::Result<()> {
    add_row_in_table(mod_context, lu_mod, lu_mod.mod_type.clone()) //eh
}

pub fn add_row_in_table(
    mod_context: &ModContext,
    lu_mod: &mut Mod,
    table_name: String,
) -> eyre::Result<()> {
    for src_table in mod_context.database.tables()?.iter() {
        let src_table = src_table?;
        if src_table.name() == table_name {
            let fields = make_row_fields(&src_table, &lu_mod.values)?;
            lu_mod.fields = fields;
            return Ok(());
        }
    }
    Ok(())
}

pub fn make_row_fields(
    table: &assembly_fdb::mem::Table,
    values: &HashMap<String, serde_json::Value>,
) -> eyre::Result<Vec<Field>> {
    let mut fields = Vec::with_capacity(table.column_count());
    for column in table.column_iter() {
        if values.contains_key(&column.name().to_string()) {
            let value_type = column.value_type();
            let value = values.get(&column.name().to_string()).unwrap();
            let field = match value_type {
                ValueType::Boolean => Field::Boolean(value.as_bool().unwrap()),
                ValueType::Integer => Field::Integer(value.as_i64().unwrap() as i32),
                ValueType::BigInt => Field::BigInt(value.as_i64().unwrap()),
                ValueType::Float => Field::Float(value.as_f64().unwrap() as f32),
                ValueType::Text => Field::Text(value.as_str().unwrap().to_string()),
                ValueType::VarChar => Field::Text(value.as_str().unwrap().to_string()),
                ValueType::Nothing => Field::Nothing,
            };
            fields.push(field);
        } else {
            fields.push(assembly_fdb::core::Field::Nothing);
        }
    }

    Ok(fields)
}

pub fn get_table<'a>(
    database: &'a assembly_fdb::mem::Database,
    name: &str,
) -> eyre::Result<assembly_fdb::mem::Table<'a>> {
    for table in database.tables()?.iter() {
        let table = table?;
        if table.name() == name {
            return Ok(table);
        }
    }
    Err(eyre!("Table {} not found", name))
}

pub fn find_available_ids(
    table: &assembly_fdb::mem::Table,
    count: usize,
) -> eyre::Result<Vec<i32>> {
    let ids = table
        .row_iter()
        .map(|row| {
            let id = row.field_at(0).unwrap();
            if let assembly_fdb::mem::Field::Integer(id) = id {
                Ok(id)
            } else {
                Err(eyre!("Non-integer id in {}", table.name()))
            }
        })
        .collect::<Result<Vec<_>, eyre::Error>>()?;

    let mut available_ids = Vec::with_capacity(count);
    let mut potential_id = 1;
    while available_ids.len() < count {
        if !ids.contains(&potential_id) {
            available_ids.push(potential_id);
        }
        potential_id += 1;
    }
    Ok(available_ids)
}

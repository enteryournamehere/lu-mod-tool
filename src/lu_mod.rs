#![allow(unused_variables)]

use crate::mod_type_to_table_name;
use crate::ModContext;
use crate::Phrase;
use crate::Translation;
use assembly_fdb::common::ValueType;
use assembly_fdb::core::Field;
use color_eyre::eyre::{self, eyre};
use serde::{Deserialize, Serialize};
use serde_json::{to_value as to_json_value, Value as JsonValue};
use std::collections::HashMap;
use std::io::Read;
use std::path::PathBuf;

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
    #[serde(default)]
    skills: Vec<JsonValue>,
    #[serde(default)]
    tasks: Vec<MissionTask>,
    #[serde(default)]
    missions: Vec<MissionOffer>,
    pub locale: HashMap<String, String>,
    pub values: HashMap<String, serde_json::Value>,
    #[serde(default, skip)]
    pub output_values: HashMap<String, OutputValue>,
    #[serde(skip)]
    pub defaults: HashMap<String, Field>,
    #[serde(skip)]
    pub fields: Vec<OutputValue>,
    #[serde(skip)]
    pub dir: PathBuf,
    #[serde(skip)]
    pub new_locale_entries: Vec<Phrase>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MissionOffer {
    pub mission: String,
    pub accept: bool,
    pub offer: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MissionTask {
    #[serde(rename = "type")]
    pub task_type: String,
    pub target: JsonValue,
    pub count: i32,
    pub group: Vec<JsonValue>,
    #[serde(rename = "location")]
    pub target_group_string: Option<String>,
    pub parameters: Option<String>,
    pub icon: String,
    #[serde(rename = "small-icon")]
    pub small_icon: String,
    pub locale: HashMap<String, String>,
}

impl Mod {
    fn set_default<T>(&mut self, key: &str, value: T) -> eyre::Result<()>
    where
        T: serde::Serialize,
    {
        if self.values.get(key).is_none() {
            let value = to_json_value(value)?;
            self.output_values
                .insert(key.to_string(), OutputValue::FromJson(value));
        }
        Ok(())
    }

    fn set_value<T>(&mut self, key: &str, value: T) -> eyre::Result<()>
    where
        T: serde::Serialize,
    {
        let value = to_json_value(value)?;
        self.output_values
            .insert(key.to_string(), OutputValue::FromJson(value));
        Ok(())
    }

    fn set_to_be_generated(&mut self, key: &str) -> eyre::Result<()> {
        self.output_values
            .insert(key.to_string(), OutputValue::GenerateId);
        Ok(())
    }

    fn set_awaiting_id(&mut self, key: &str, id_string: &str) -> eyre::Result<()> {
        self.output_values.insert(
            key.to_string(),
            OutputValue::AwaitingId(id_string.to_string()),
        );
        Ok(())
    }

    pub fn init_output_values(&mut self) {
        for (key, value) in self.values.iter() {
            self.output_values
                .insert(key.to_string(), OutputValue::FromJson(value.clone()));
        }
    }

    /// Generate the fields for DB insertion for this mod.
    fn set_fields(&mut self, mod_context: &ModContext) -> eyre::Result<()> {
        let table_name = mod_type_to_table_name(self.mod_type.as_str());
        for src_table in mod_context.database.tables()?.iter() {
            let src_table = src_table?;
            if src_table.name() == table_name {
                let mut fields = make_row_fields(&src_table, &self.output_values)?;
                // run all Field::Texts in fields through convert_path_specifier
                for field in fields.iter_mut() {
                    if let OutputValue::Known(Field::Text(ref mut text)) = field {
                        *text = convert_path_specifier(self, text);
                    }
                }
                self.fields = fields;
                return Ok(());
            }
        }
        Ok(())
    }

    pub fn get_target_table_name(&self) -> String {
        mod_type_to_table_name(&self.mod_type)
    }

    /// Create component, register it in the mod_context and link it to this mod.
    pub fn add_component(
        &mut self,
        mod_context: &mut ModContext,
        component_type: &str,
    ) -> eyre::Result<Mod> {
        let id_str = format!("{}:{}", self.id, component_type);
        self.components.push(id_str.clone());
        let mut output = Mod {
            id: id_str,
            mod_type: component_type.to_string(),
            ..self.clone()
        };
        apply_component_mod(mod_context, &mut output)?;
        mod_context.mods.push(output.clone());
        Ok(output)
    }

    pub fn add_locale(&mut self, phrase_id: &str) {
        if !self.locale.is_empty() {
            let phrase = Phrase {
                id: phrase_id.to_string(),
                translations: self
                    .locale
                    .iter()
                    .map(|(k, v)| Translation {
                        locale: k.clone(),
                        value: v.clone(),
                    })
                    .collect::<Vec<Translation>>(),
            };
            self.new_locale_entries.push(phrase);
        }
    }

    pub fn add_locale_from_value(&mut self, phrase_id: &str, key: &str) {
        if let Some(serde_json::Value::Object(value)) = self.values.get(key) {
            let phrase = Phrase {
                id: phrase_id.to_string(),
                translations: value
                    .iter()
                    .map(|(k, v)| Translation {
                        locale: k.clone(),
                        value: v.as_str().unwrap().to_string(), // will crash if bad
                    })
                    .collect::<Vec<Translation>>(),
            };
            self.new_locale_entries.push(phrase);
        }
    }
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
            skills: vec![],
            tasks: vec![],
            missions: vec![],
            locale: HashMap::new(),
            values: HashMap::new(),
            output_values: HashMap::new(),
            defaults: HashMap::new(),
            fields: vec![],
            dir: PathBuf::new(),
            new_locale_entries: vec![],
        }
    }
}

#[derive(Debug, Clone)]
pub enum OutputValue {
    Known(Field),
    FromJson(JsonValue),
    AwaitingId(String),
    GenerateId,
}

pub fn convert_path_specifier(lu_mod: &Mod, contents: &str) -> String {
    if let Some(asset_path) = contents.strip_prefix("ASSET:") {
        let mut relative_path_to_mods = "../mods";
        let mut relative_path_from_mods = asset_path;

        if let Some(physics_path) = asset_path.strip_prefix("PHYSICS:") {
            relative_path_to_mods = "../../mods";
            relative_path_from_mods = physics_path;
        } else if let Some(icon_path) = asset_path.strip_prefix("ICON:") {
            relative_path_to_mods = "../../../mods";
            relative_path_from_mods = icon_path;
        }

        let path = PathBuf::from(relative_path_to_mods)
            .join(&lu_mod.dir)
            .join(relative_path_from_mods);

        // use backslashes as path separators
        return path.to_str().unwrap().to_string().replace("/", "\\");
    }
    contents.to_string()
}

pub fn apply_sql_mod(_mod_context: &ModContext, lu_mod: &mut Mod) -> eyre::Result<()> {
    if let Some(sql) = &lu_mod.values.get("sql") {
        if let Some(sql_str) = sql.as_str() {
            if let Some(path) = sql_str.strip_prefix("INCLUDE:") {
                // load from path
                let mut sql_file = std::fs::File::open(lu_mod.dir.join(path))?;
                let mut sql_str = String::new();
                sql_file.read_to_string(&mut sql_str)?;
                lu_mod.values.insert(String::from("sql"), sql_str.into());
                return Ok(());
            } else {
                let sql_value = sql_str.into();
                lu_mod.values.insert(String::from("sql"), sql_value);
            }
            return Ok(());
        } else {
            return Err(eyre!("incorrect value type for sql"));
        }
    }

    Err(eyre!("sql not set"))
}

pub fn apply_item_mod(mod_context: &mut ModContext, lu_mod: &mut Mod) -> eyre::Result<()> {
    lu_mod.set_default("nametag", false)?;
    lu_mod.set_default("localize", true)?;
    lu_mod.set_default("locStatus", 2)?;
    lu_mod.set_default("offsetGroupID", 78)?;
    lu_mod.set_default("itemInfo", 0)?;
    lu_mod.set_default("fade", true)?;
    lu_mod.set_default("fadeInTime", 1)?;
    lu_mod.set_default("shader_id", 23)?;
    lu_mod.set_default("audioEquipMetaEventSet", "Weapon_Hammer_Generic")?;
    lu_mod.set_value("type", "Loot")?;

    lu_mod.add_component(mod_context, "ItemComponent")?;
    lu_mod.add_component(mod_context, "RenderComponent")?;

    if !lu_mod.skills.is_empty() {
        lu_mod.add_component(mod_context, "SkillComponent")?;

        for (index, skill) in lu_mod.skills.iter().enumerate() {
            let mut object_skills_mod = Mod {
                id: lu_mod.id.clone() + ":skills:" + index.to_string().as_str(),
                mod_type: "ObjectSkills".to_string(),
                dir: lu_mod.dir.clone(),
                ..Default::default()
            };
            object_skills_mod.set_value(
                "castOnType",
                if let Some(cast_on_type) = lu_mod.values.get("castOnType") {
                    cast_on_type.as_i64().unwrap()
                } else {
                    0
                },
            )?;
            object_skills_mod.set_value("AICombatWeight", 0)?;
            object_skills_mod.set_awaiting_id("objectTemplate", &lu_mod.id)?;

            if skill.is_number() {
                object_skills_mod.set_value("skillID", skill.as_i64().unwrap())?;
            } else {
                object_skills_mod.set_awaiting_id("skillID", skill.as_str().unwrap())?;
            }

            object_skills_mod.set_fields(mod_context)?;

            mod_context.mods.push(object_skills_mod);
        }
    }
    apply_object_mod(mod_context, lu_mod)?;

    Ok(())
}

pub fn apply_environmental_mod(mod_context: &mut ModContext, lu_mod: &mut Mod) -> eyre::Result<()> {
    lu_mod.set_default("static", 1)?;
    lu_mod.set_default("shader_id", 1)?;
    lu_mod.set_value("type", "Environmental")?;

    lu_mod.add_component(mod_context, "RenderComponent")?;
    lu_mod.add_component(mod_context, "SimplePhysicsComponent")?;

    apply_object_mod(mod_context, lu_mod)?;

    Ok(())
}

pub fn apply_mission_mod(mod_context: &mut ModContext, lu_mod: &mut Mod) -> eyre::Result<()> {
    lu_mod.set_default("locStatus", 2)?;
    lu_mod.set_default("UIPrereqID", JsonValue::Null)?;
    lu_mod.set_default("localize", true)?;
    lu_mod.set_default("isMission", true)?;
    lu_mod.set_default("isChoiceReward", false)?;
    lu_mod.set_default("missionIconID", JsonValue::Null)?;
    lu_mod.set_default("time_limit", JsonValue::Null)?;
    lu_mod.set_default("reward_item1", -1)?;
    lu_mod.set_default("reward_item2", -1)?;
    lu_mod.set_default("reward_item3", -1)?;
    lu_mod.set_default("reward_item4", -1)?;
    lu_mod.set_default("reward_item1_repeatable", -1)?;
    lu_mod.set_default("reward_item2_repeatable", -1)?;
    lu_mod.set_default("reward_item3_repeatable", -1)?;
    lu_mod.set_default("reward_item4_repeatable", -1)?;
    lu_mod.set_default("reward_emote", -1)?;
    lu_mod.set_default("reward_emote2", -1)?;
    lu_mod.set_default("reward_emote3", -1)?;
    lu_mod.set_default("reward_emote4", -1)?;
    lu_mod.set_default("reward_maxwallet", 0)?;
    lu_mod.set_default("reward_reputation", 0)?;
    lu_mod.set_default("reward_currency_repeatable", 0)?;
    lu_mod.set_to_be_generated("id")?;

    lu_mod.set_fields(mod_context)?;

    // Locale
    lu_mod.add_locale("Missions_{}_name");
    lu_mod.add_locale_from_value("MissionText_{}_accept_chat_bubble", "accept_chat_bubble");
    lu_mod.add_locale_from_value("MissionText_{}_accept_chat_bubble", "chat_accept");
    lu_mod.add_locale_from_value("MissionText_{}_chat_state_1", "chat_state_1");
    lu_mod.add_locale_from_value("MissionText_{}_chat_state_2", "chat_state_2");
    lu_mod.add_locale_from_value("MissionText_{}_chat_state_3", "chat_state_3");
    lu_mod.add_locale_from_value("MissionText_{}_chat_state_4", "chat_state_4");
    lu_mod.add_locale_from_value("MissionText_{}_chat_state_1", "chat_available");
    lu_mod.add_locale_from_value("MissionText_{}_chat_state_2", "chat_active");
    lu_mod.add_locale_from_value("MissionText_{}_chat_state_3", "chat_ready_to_complete");
    lu_mod.add_locale_from_value("MissionText_{}_chat_state_4", "chat_complete");
    lu_mod.add_locale_from_value(
        "MissionText_{}_completion_succeed_tip",
        "completion_succeed_tip",
    );
    lu_mod.add_locale_from_value("MissionText_{}_in_progress", "in_progress");
    lu_mod.add_locale_from_value("MissionText_{}_offer", "offer");
    lu_mod.add_locale_from_value("MissionText_{}_ready_to_complete", "ready_to_complete");

    // MissionText entry
    let mut mission_text_mod = Mod {
        id: lu_mod.id.clone() + ":MissionText",
        mod_type: "MissionText".to_string(),
        dir: lu_mod.dir.clone(),
        ..Default::default()
    };

    // Set values
    mission_text_mod.set_value("localize", true)?;
    mission_text_mod.set_value("locStatus", 2)?;
    mission_text_mod.set_awaiting_id("id", &lu_mod.id)?;

    // Convert to output fields
    mission_text_mod.set_fields(mod_context)?;

    mod_context.mods.push(mission_text_mod);

    // Mission Tasks
    for (index, task) in lu_mod.tasks.iter().enumerate() {
        let mut task_mod = Mod {
            id: lu_mod.id.clone() + ":tasks:" + index.to_string().as_str(),
            mod_type: "MissionTasks".to_string(),
            locale: task.locale.clone(),
            output_values: lu_mod.output_values.clone(),
            dir: lu_mod.dir.clone(),
            ..Default::default()
        };
        task_mod.set_value("taskType", 0)?; // TODO: read & convert
        task_mod.set_value("target", task.target.clone())?;
        task_mod.set_value("targetValue", task.count)?;
        task_mod.set_awaiting_id("id", &lu_mod.id)?;
        task_mod.set_to_be_generated("uid")?;
        if let Some(target_group_string) = &task.target_group_string {
            task_mod.set_value("targetGroup", target_group_string.clone())?;
        } else {
            let values = &task.group;
            // join with commas
            let group_string = values
                .iter()
                .map(|v| v.to_string())
                .collect::<Vec<String>>()
                .join(",");
            task_mod.set_value("targetGroup", group_string)?;
        }

        task_mod.set_fields(mod_context)?;

        task_mod.add_locale("MissionTasks_{}_description");

        mod_context.mods.push(task_mod);
    }

    Ok(())
}

pub fn apply_npc_mod(mod_context: &mut ModContext, lu_mod: &mut Mod) -> eyre::Result<()> {
    lu_mod.set_default("render_asset", "animations\\\\minifig\\\\mf_ambient.kfm")?;
    lu_mod.set_default("animationGroupIDs", "93")?;
    lu_mod.set_default("shader_id", 14)?;
    lu_mod.set_default("static", 1)?;
    lu_mod.set_default("jump", 0)?;
    lu_mod.set_default("doublejump", 0)?;
    lu_mod.set_default("speed", 5)?;
    lu_mod.set_default("rotSpeed", 360)?;
    lu_mod.set_default("playerHeight", 4.4)?;
    lu_mod.set_default("playerRadius", 1)?;
    lu_mod.set_default("pcShapeType", 2)?;
    lu_mod.set_default("collisionGroup", 3)?;
    lu_mod.set_default("airSpeed", 5)?;
    lu_mod.set_default("jumpAirSpeed", 25)?;
    lu_mod.set_default("interactionDistance", JsonValue::Null)?;

    lu_mod.set_default("chatBubbleOffset", JsonValue::Null)?;
    lu_mod.set_default("fade", true)?;
    lu_mod.set_default("fadeInTime", 1)?;
    lu_mod.set_default("billboardHeight", JsonValue::Null)?;
    lu_mod.set_default("AudioMetaEventSet", "Emotes_Non_Player")?;
    lu_mod.set_default("usedropshadow", false)?;
    lu_mod.set_default("preloadAnimations", false)?;
    lu_mod.set_default("ignoreCameraCollision", false)?;
    lu_mod.set_default("gradualSnap", false)?;
    lu_mod.set_default("staticBillboard", false)?;
    lu_mod.set_default("attachIndicatorsToNode", false)?;

    lu_mod.set_default("npcTemplateID", 14)?;
    lu_mod.set_default("nametag", true)?;
    lu_mod.set_default("placeable", true)?;
    lu_mod.set_default("localize", true)?;
    lu_mod.set_default("locStatus", 2)?;

    lu_mod.set_value("type", "UserGeneratedNPCs")?;

    lu_mod.add_component(mod_context, "SimplePhysicsComponent")?;
    lu_mod.add_component(mod_context, "RenderComponent")?;
    lu_mod.add_component(mod_context, "MinifigComponent")?;

    // to-do items
    if !lu_mod.missions.is_empty() {
        let first_id = lu_mod.id.clone() + ":MissionNPCComponent:0";
        for (index, mission) in lu_mod.missions.iter().enumerate() {
            let component_id = if index == 0 {
                first_id.clone()
            } else {
                lu_mod.id.clone() + ":MissionNPCComponent:" + index.to_string().as_str()
            };
            let mut mission_npc_component = Mod {
                id: component_id.clone(),
                mod_type: "MissionNPCComponent".to_string(),
                dir: lu_mod.dir.clone(),
                output_values: lu_mod.output_values.clone(),
                ..Default::default()
            };
            if index == 0 {
                mission_npc_component.set_to_be_generated("id")?;
            } else {
                mission_npc_component.set_awaiting_id("id", first_id.as_str())?;
            }
            mission_npc_component.set_value("missionID", mission.mission.clone())?;
            mission_npc_component.set_value("offersMission", mission.offer)?;
            mission_npc_component.set_value("acceptsMission", mission.accept)?;

            mission_npc_component.set_fields(mod_context)?;

            mod_context.mods.push(mission_npc_component);
        }
        lu_mod.components.push(first_id);
    }

    lu_mod.set_to_be_generated("id")?;

    apply_object_mod(mod_context, lu_mod)?;

    Ok(())
}

pub fn apply_object_mod(mod_context: &mut ModContext, lu_mod: &mut Mod) -> eyre::Result<()> {
    lu_mod.add_locale("Objects_{}_name");

    lu_mod.set_to_be_generated("id")?;
    lu_mod.set_fields(mod_context)
}

pub fn apply_component_mod(mod_context: &ModContext, lu_mod: &mut Mod) -> eyre::Result<()> {
    lu_mod.set_to_be_generated("id")?;
    lu_mod.set_fields(mod_context)
}

pub fn make_row_fields(
    table: &assembly_fdb::mem::Table,
    values: &HashMap<String, OutputValue>,
) -> eyre::Result<Vec<OutputValue>> {
    let mut fields = Vec::with_capacity(table.column_count());
    for column in table.column_iter() {
        let fieldy = if values.contains_key(&column.name().to_string()) {
            let value_type = column.value_type();
            let value = values.get(&column.name().to_string()).unwrap();

            match value {
                OutputValue::FromJson(json_value) => {
                    if json_value == &JsonValue::Null {
                        OutputValue::Known(Field::Nothing)
                    } else {
                        let field = match value_type {
                            ValueType::Boolean => {
                                OutputValue::Known(Field::Boolean(json_value.as_bool().unwrap()))
                            }
                            ValueType::Integer => {
                                if let Some(as_i64) = json_value.as_i64() {
                                    OutputValue::Known(Field::Integer(as_i64 as i32))
                                } else {
                                    OutputValue::AwaitingId(
                                        json_value.as_str().unwrap().to_string(),
                                    )
                                }
                            }
                            ValueType::BigInt => {
                                OutputValue::Known(Field::BigInt(json_value.as_i64().unwrap()))
                            }
                            ValueType::Float => OutputValue::Known(Field::Float(
                                json_value.as_f64().unwrap() as f32,
                            )),
                            ValueType::Text => OutputValue::Known(Field::Text(
                                json_value.as_str().unwrap().to_string(),
                            )),
                            ValueType::VarChar => OutputValue::Known(Field::Text(
                                json_value.as_str().unwrap().to_string(),
                            )),
                            ValueType::Nothing => OutputValue::Known(Field::Nothing),
                        };
                        field
                    }
                }
                _ => value.clone(), // ??
            }
        } else {
            OutputValue::Known(Field::Nothing)
        };
        fields.push(fieldy);
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

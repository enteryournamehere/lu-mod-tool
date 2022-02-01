use crate::HashMap;
use color_eyre::eyre::{self, eyre};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;

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

pub fn parse_mission_task_type(input: &str) -> eyre::Result<i32> {
    match input {
        "Smash" => Ok(0),
        "Script" => Ok(1),
        "Activity" => Ok(2),
        "Environment" => Ok(3),
        "MissionInteraction" => Ok(4),
        "Emote" => Ok(5),
        "Food" => Ok(9),
        "Skill" => Ok(10),
        "ItemCollection" => Ok(11),
        "Location" => Ok(12),
        "Minigame" => Ok(14),
        "NonMissionInteraction" => Ok(15),
        "MissionComplete" => Ok(16),
        "Powerup" => Ok(21),
        "PetTaming" => Ok(22),
        "Racing" => Ok(23),
        "PlayerFlag" => Ok(24),
        "VisitProperty" => Ok(30),
        _ => {
            return Err(eyre!("Unknown mission task type: {}", input));
        }
    }
}

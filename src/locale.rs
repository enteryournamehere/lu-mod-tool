use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Localization {
    version: f32,
    locales: Locales,
    phrases: Phrases,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Locales {
    count: i32,
    locale: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Phrases {
    count: i32,
    phrase: Vec<Phrase>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Phrase {
    id: String,
    #[serde(rename = "translation")]
    translations: Vec<Translation>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Translation {
    locale: String,
    #[serde(rename = "$value", default)]
    text: String,
}

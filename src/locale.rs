use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename = "localization")]
pub struct Localization {
    pub version: f32,
    pub locales: Locales,
    pub phrases: Phrases,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Locales {
    pub count: i32,
    pub locale: Vec<Locale>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Locale {
    #[serde(rename = "$value", default)]
    pub value: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Phrases {
    pub count: i32,
    pub phrase: Vec<Phrase>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Phrase {
    pub id: String,
    #[serde(rename = "translation")]
    pub translations: Vec<Translation>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Translation {
    pub locale: String,
    #[serde(rename = "$value", default)]
    pub value: String,
}

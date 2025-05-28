use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GameConfig {
    pub game_info: GameInfo,
    pub biomes: HashMap<String, BiomeConfig>,
    pub starting_biome: String,
    pub player_config: PlayerConfig,
    pub items: HashMap<String, ItemConfig>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GameInfo {
    pub name: String,
    pub version: String,
    pub description: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BiomeConfig {
    pub name: String,
    pub map_file: String,
    pub monster_file: String,
    pub sprite_file: String,
    pub music: String,
    pub ambient_color: String,
    pub weather: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PlayerConfig {
    pub starting_hp: i32,
    pub starting_attack: i32,
    pub level_up_hp_bonus: i32,
    pub level_up_attack_bonus: i32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ItemConfig {
    pub name: String,
    #[serde(rename = "type")]
    pub item_type: String,
    #[serde(default)]
    pub effect: Option<String>,
    #[serde(default)]
    pub value: Option<i32>,
    #[serde(default)]
    pub attack_bonus: Option<i32>,
    #[serde(default)]
    pub defense_bonus: Option<i32>,
    pub price: i32,
}

pub fn load_game_config(path: &str) -> Result<GameConfig, String> {
    match std::fs::read_to_string(path) {
        Ok(contents) => {
            match serde_json::from_str::<GameConfig>(&contents) {
                Ok(config) => Ok(config),
                Err(e) => Err(format!("JSON parsing error: {}", e)),
            }
        },
        Err(e) => Err(format!("File read error: {}", e)),
    }
}
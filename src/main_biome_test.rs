mod sprite_system;
mod biome_system;

use macroquad::prelude::*;
use serde::{Deserialize, Serialize};
use std::fs;
use std::collections::HashMap;
use sprite_system::{SpriteRenderer, load_sprites};
use biome_system::{GameConfig, load_game_config};

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Monster {
    name: String,
    species: String,
    hp: i32,
    attack: i32,
    speed: f32,
    color: String,
    behavior: Vec<String>,
    loot: LootData,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct LootData {
    gold: i32,
    item_chance: f32,
}

#[derive(Serialize, Deserialize, Debug)]
struct MonsterData {
    monsters: Vec<Monster>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct MapData {
    map_info: MapInfo,
    tile_types: HashMap<String, TileType>,
    layout: Vec<String>,
    monster_spawns: Option<Vec<MonsterSpawn>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct MapInfo {
    name: String,
    width: usize,
    height: usize,
    tile_size: f32,
    spawn_point: SpawnPoint,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct SpawnPoint {
    x: f32,
    y: f32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct TileType {
    name: String,
    walkable: bool,
    color: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct MonsterSpawn {
    x: f32,
    y: f32,
    monster_type: String,
}

struct BiomeData {
    map_data: MapData,
    monsters: Vec<Monster>,
    sprite_renderer: SpriteRenderer,
}

fn parse_color(color_str: &str) -> Color {
    if color_str.starts_with('#') {
        let hex = &color_str[1..];
        if hex.len() == 6 {
            if let Ok(r) = u8::from_str_radix(&hex[0..2], 16) {
                if let Ok(g) = u8::from_str_radix(&hex[2..4], 16) {
                    if let Ok(b) = u8::from_str_radix(&hex[4..6], 16) {
                        return Color::new(r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0, 1.0);
                    }
                }
            }
        }
    }
    
    match color_str {
        "red" => RED,
        "green" => GREEN,
        "darkgreen" => Color::new(0.0, 0.5, 0.0, 1.0),
        "blue" => BLUE,
        "yellow" => YELLOW,
        "brown" => Color::new(0.5, 0.25, 0.0, 1.0),
        "orange" => ORANGE,
        _ => WHITE,
    }
}

#[macroquad::main("Biome Test System")]
async fn main() {
    // Load game configuration
    let game_config = match load_game_config("data/game_config.json") {
        Ok(config) => {
            println!("Game config loaded: {}", config.game_info.name);
            config
        },
        Err(e) => {
            println!("Failed to load game config: {}", e);
            return;
        }
    };
    
    // Load biomes
    let mut biomes: HashMap<String, BiomeData> = HashMap::new();
    let mut current_biome = "forest".to_string();
    
    // Load forest biome
    if let Some(forest_config) = game_config.biomes.get("forest") {
        if let Ok(biome) = load_biome(forest_config) {
            biomes.insert("forest".to_string(), biome);
        }
    }
    
    // Load desert biome
    if let Some(desert_config) = game_config.biomes.get("desert") {
        if let Ok(biome) = load_biome(desert_config) {
            biomes.insert("desert".to_string(), biome);
        }
    }
    
    let mut player_pos = vec2(400.0, 300.0);
    
    loop {
        clear_background(BLACK);
        
        // Get current biome
        if let Some(biome) = biomes.get(&current_biome) {
            // Draw map
            let tile_size = biome.map_data.map_info.tile_size;
            for (y, row) in biome.map_data.layout.iter().enumerate() {
                for (x, tile_char) in row.chars().enumerate() {
                    let tile_key = tile_char.to_string();
                    if let Some(tile_type) = biome.map_data.tile_types.get(&tile_key) {
                        let pos = vec2(x as f32 * tile_size, y as f32 * tile_size);
                        draw_rectangle(
                            pos.x,
                            pos.y,
                            tile_size,
                            tile_size,
                            parse_color(&tile_type.color)
                        );
                    }
                }
            }
            
            // Display biome info
            draw_text(&format!("Current Biome: {}", biome.map_data.map_info.name), 10.0, 30.0, 30.0, WHITE);
            draw_text(&format!("Monsters: {} types", biome.monsters.len()), 10.0, 60.0, 20.0, WHITE);
            
            // List monsters
            draw_text("Monster Types:", 10.0, 100.0, 20.0, YELLOW);
            for (i, monster) in biome.monsters.iter().enumerate() {
                draw_text(
                    &format!("- {} (HP: {}, ATK: {})", monster.name, monster.hp, monster.attack),
                    10.0,
                    130.0 + i as f32 * 25.0,
                    18.0,
                    WHITE
                );
            }
        }
        
        // Controls
        draw_text("Press 1 for Forest, 2 for Desert", 10.0, screen_height() - 60.0, 20.0, GREEN);
        draw_text("Press Q to quit", 10.0, screen_height() - 30.0, 20.0, GREEN);
        
        // Switch biomes
        if is_key_pressed(KeyCode::Key1) {
            current_biome = "forest".to_string();
            println!("Switched to Forest biome");
        }
        if is_key_pressed(KeyCode::Key2) {
            current_biome = "desert".to_string();
            println!("Switched to Desert biome");
        }
        
        if is_key_pressed(KeyCode::Q) {
            break;
        }
        
        next_frame().await
    }
}

fn load_biome(config: &biome_system::BiomeConfig) -> Result<BiomeData, String> {
    // Load map
    let map_data = load_map(&config.map_file)?;
    
    // Load monsters
    let monsters = load_monsters(&config.monster_file)?;
    
    // Load sprites
    let sprite_data = load_sprites(&config.sprite_file)?;
    let sprite_renderer = SpriteRenderer::new(sprite_data);
    
    Ok(BiomeData {
        map_data,
        monsters,
        sprite_renderer,
    })
}

fn load_map(path: &str) -> Result<MapData, String> {
    match fs::read_to_string(path) {
        Ok(contents) => {
            match serde_yaml::from_str::<MapData>(&contents) {
                Ok(data) => Ok(data),
                Err(e) => Err(format!("Map YAML parsing error: {}", e)),
            }
        },
        Err(e) => Err(format!("Map file read error: {}", e)),
    }
}

fn load_monsters(path: &str) -> Result<Vec<Monster>, String> {
    match fs::read_to_string(path) {
        Ok(contents) => {
            match serde_yaml::from_str::<MonsterData>(&contents) {
                Ok(data) => Ok(data.monsters),
                Err(e) => Err(format!("Monster YAML parsing error: {}", e)),
            }
        },
        Err(e) => Err(format!("Monster file read error: {}", e)),
    }
}
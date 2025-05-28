mod sprite_system;
mod biome_system;
mod chunk_system;

use macroquad::prelude::*;
use serde::{Deserialize, Serialize};
use std::fs;
use std::collections::HashMap;
use sprite_system::{SpriteRenderer, load_sprites};
use chunk_system::{ChunkManager, load_world_config};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Monster {
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
pub struct LootData {
    gold: i32,
    item_chance: f32,
}

#[derive(Serialize, Deserialize, Debug)]
struct MonsterData {
    monsters: Vec<Monster>,
}

#[derive(Clone)]
pub struct ActiveMonster {
    pub data: Monster,
    pub position: Vec2,
    pub current_hp: i32,
    pub is_dead: bool,
    pub respawn_timer: f32,
}

struct Player {
    position: Vec2,
    hp: i32,
    max_hp: i32,
    attack: i32,
    radius: f32,
    attack_cooldown: f32,
    is_attacking: bool,
    facing_left: bool,
    animation_timer: f32,
}

struct DamageText {
    position: Vec2,
    text: String,
    timer: f32,
    color: Color,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MapData {
    pub map_info: MapInfo,
    pub tile_types: HashMap<String, TileType>,
    pub layout: Vec<String>,
    pub monster_spawns: Option<Vec<MonsterSpawn>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MapInfo {
    pub name: String,
    pub width: usize,
    pub height: usize,
    pub tile_size: f32,
    pub spawn_point: SpawnPoint,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SpawnPoint {
    pub x: f32,
    pub y: f32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TileType {
    pub name: String,
    pub walkable: bool,
    pub color: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MonsterSpawn {
    pub x: f32,
    pub y: f32,
    pub monster_type: String,
}

struct Camera {
    position: Vec2,
}

pub fn string_to_color(color_str: &str) -> Color {
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
        "green" => GREEN,
        "darkgreen" => Color::new(0.0, 0.5, 0.0, 1.0),
        "red" => RED,
        "blue" => BLUE,
        "purple" => PURPLE,
        "yellow" => YELLOW,
        "gray" => GRAY,
        "lightgray" => LIGHTGRAY,
        "darkgray" => DARKGRAY,
        "brown" => Color::new(0.5, 0.25, 0.0, 1.0),
        "pink" => PINK,
        _ => WHITE,
    }
}

fn check_collision(pos1: Vec2, radius1: f32, pos2: Vec2, radius2: f32) -> bool {
    (pos1 - pos2).length() < radius1 + radius2
}

#[macroquad::main("Seamless RPG World")]
async fn main() {
    // Load world configuration
    let world_config = match load_world_config("data/world_config.json") {
        Ok(config) => {
            println!("World config loaded: {}", config.world_info.name);
            config
        },
        Err(e) => {
            println!("Failed to load world config: {}", e);
            return;
        }
    };
    
    // Load all monster templates
    let mut all_monsters = Vec::new();
    all_monsters.extend(load_monsters("data/monsters/forest_monsters_en.yaml"));
    all_monsters.extend(load_monsters("data/monsters/desert_monsters.yaml"));
    println!("Loaded {} monster types", all_monsters.len());
    
    // Load sprite data
    let mut sprite_renderers = HashMap::new();
    
    // Load forest sprites
    if let Ok(sprite_data) = load_sprites("data/sprites/character_sprites.json") {
        sprite_renderers.insert("forest".to_string(), SpriteRenderer::new(sprite_data));
    }
    
    // Load desert sprites
    if let Ok(sprite_data) = load_sprites("data/sprites/desert_sprites.json") {
        sprite_renderers.insert("desert".to_string(), SpriteRenderer::new(sprite_data));
    }
    
    // Create chunk manager
    let mut chunk_manager = ChunkManager::new(world_config.clone());
    chunk_manager.initialize(&all_monsters);
    
    // Initialize player at spawn position
    let spawn_chunk_offset = chunk_manager.chunk_to_world_coords(1, 1); // Central chunk
    let spawn_pos = spawn_chunk_offset + vec2(
        world_config.spawn_position.x * 32.0,
        world_config.spawn_position.y * 32.0
    );
    
    let mut player = Player {
        position: spawn_pos,
        hp: 100,
        max_hp: 100,
        attack: 15,
        radius: 10.0,
        attack_cooldown: 0.0,
        is_attacking: false,
        facing_left: false,
        animation_timer: 0.0,
    };
    
    let mut damage_texts: Vec<DamageText> = Vec::new();
    let mut gold_collected = 0;
    let mut game_over = false;
    let mut camera = Camera { position: player.position };
    
    loop {
        clear_background(BLACK);
        
        let delta = get_frame_time();
        
        // Update camera to follow player
        camera.position = player.position;
        
        // Update loaded chunks based on player position
        chunk_manager.update_loaded_chunks(player.position, &all_monsters);
        
        // Player movement input
        let mut move_dir = vec2(0.0, 0.0);
        if !game_over {
            if is_key_down(KeyCode::W) { move_dir.y -= 1.0; }
            if is_key_down(KeyCode::S) { move_dir.y += 1.0; }
            if is_key_down(KeyCode::A) { move_dir.x -= 1.0; }
            if is_key_down(KeyCode::D) { move_dir.x += 1.0; }
            
            if move_dir.length() > 0.0 {
                // Update facing direction
                if move_dir.x < 0.0 {
                    player.facing_left = true;
                } else if move_dir.x > 0.0 {
                    player.facing_left = false;
                }
                
                // Update animation timer
                player.animation_timer += delta * 5.0;
                
                let normalized_move = move_dir.normalize();
                let move_speed = 200.0 * delta;
                let desired_pos = player.position + normalized_move * move_speed;
                
                // Check if we can move to the desired position
                let check_position = |pos: Vec2| -> bool {
                    let corners = [
                        pos + vec2(-player.radius, -player.radius),
                        pos + vec2(player.radius, -player.radius),
                        pos + vec2(-player.radius, player.radius),
                        pos + vec2(player.radius, player.radius),
                    ];
                    
                    for corner in &corners {
                        if !chunk_manager.is_position_walkable(*corner) {
                            return false;
                        }
                    }
                    true
                };
                
                // Try to move to the desired position with wall sliding
                if check_position(desired_pos) {
                    player.position = desired_pos;
                } else {
                    let mut final_pos = player.position;
                    
                    // Try horizontal movement
                    if move_dir.x != 0.0 {
                        let horizontal_pos = vec2(
                            player.position.x + normalized_move.x * move_speed,
                            player.position.y
                        );
                        if check_position(horizontal_pos) {
                            final_pos.x = horizontal_pos.x;
                        }
                    }
                    
                    // Try vertical movement
                    if move_dir.y != 0.0 {
                        let vertical_pos = vec2(
                            final_pos.x,
                            player.position.y + normalized_move.y * move_speed
                        );
                        if check_position(vertical_pos) {
                            final_pos.y = vertical_pos.y;
                        }
                    }
                    
                    player.position = final_pos;
                }
            }
            
            // Attack input
            if is_key_pressed(KeyCode::Space) && player.attack_cooldown <= 0.0 {
                player.is_attacking = true;
                player.attack_cooldown = 0.5;
            }
            
            // Update attack cooldown
            if player.attack_cooldown > 0.0 {
                player.attack_cooldown -= delta;
            }
            
            // Monster AI and combat - process all loaded chunks
            let chunk_ids = chunk_manager.get_loaded_chunk_ids();
            for chunk_id in chunk_ids {
                if let Some(chunk) = chunk_manager.get_chunk_mut(&chunk_id) {
                    // First, collect monster updates
                    let mut monster_updates = Vec::new();
                    for (i, monster) in chunk.active_monsters.iter().enumerate() {
                        if !monster.is_dead && monster.data.behavior.contains(&"aggressive".to_string()) {
                            let dir_to_player = player.position - monster.position;
                            if dir_to_player.length() > 0.0 && dir_to_player.length() < 300.0 {
                                let dir = dir_to_player.normalize();
                                let move_speed = monster.data.speed * 30.0 * delta;
                                let desired_pos = monster.position + dir * move_speed;
                                monster_updates.push((i, desired_pos));
                            }
                        }
                    }
                    
                    // Apply movement updates
                    for (i, desired_pos) in monster_updates {
                        if chunk_manager.is_position_walkable(desired_pos) {
                            chunk.active_monsters[i].position = desired_pos;
                        }
                    }
                }
            }
            
            // Handle combat separately
            for chunk_id in chunk_manager.get_loaded_chunk_ids() {
                if let Some(chunk) = chunk_manager.get_chunk_mut(&chunk_id) {
                    for monster in &mut chunk.active_monsters {
                        if !monster.is_dead {
                            // Check collision with player
                            let monster_radius = 15.0;
                            if check_collision(player.position, player.radius, monster.position, monster_radius) {
                                player.hp -= 1;
                                
                                if player.is_attacking {
                                    monster.current_hp -= player.attack;
                                    damage_texts.push(DamageText {
                                        position: monster.position + vec2(0.0, -30.0),
                                        text: format!("-{}", player.attack),
                                        timer: 1.0,
                                        color: YELLOW,
                                    });
                                    player.is_attacking = false;
                                    
                                    if monster.current_hp <= 0 {
                                        monster.is_dead = true;
                                        monster.respawn_timer = 5.0;
                                        gold_collected += monster.data.loot.gold;
                                        
                                        damage_texts.push(DamageText {
                                            position: monster.position,
                                            text: format!("+{} Gold", monster.data.loot.gold),
                                            timer: 2.0,
                                            color: GOLD,
                                        });
                                    }
                                }
                            }
                        } else {
                            // Handle respawn
                            monster.respawn_timer -= delta;
                            if monster.respawn_timer <= 0.0 {
                                monster.is_dead = false;
                                monster.current_hp = monster.data.hp;
                            }
                        }
                    }
                }
            }
            
            // Check game over
            if player.hp <= 0 {
                game_over = true;
            }
        }
        
        // Update damage texts
        damage_texts.retain_mut(|dt| {
            dt.timer -= delta;
            dt.position.y -= 50.0 * delta;
            dt.timer > 0.0
        });
        
        // Calculate camera offset
        let camera_offset = vec2(screen_width() / 2.0, screen_height() / 2.0) - camera.position;
        
        // Draw chunks
        let default_sprite_renderer = &sprite_renderers["forest"];
        chunk_manager.draw_chunks(camera_offset, default_sprite_renderer);
        
        // Draw player
        let player_screen_pos = player.position + camera_offset;
        let frame_name = if move_dir.length() > 0.0 {
            if player.animation_timer.sin() > 0.0 { "walk1" } else { "idle" }
        } else {
            "idle"
        };
        
        // Use appropriate sprite renderer
        let current_chunk_id = chunk_manager.get_chunk_at_position(player.position);
        let sprite_renderer = if let Some(chunk_id) = &current_chunk_id {
            if let Some(chunk_config) = world_config.chunks.get(chunk_id) {
                sprite_renderers.get(&chunk_config.biome).unwrap_or(default_sprite_renderer)
            } else {
                default_sprite_renderer
            }
        } else {
            default_sprite_renderer
        };
        
        sprite_renderer.draw_sprite_outlined(
            "player",
            frame_name,
            player_screen_pos,
            2.0,
            player.facing_left,
            BLACK
        );
        
        // Draw attack indicator
        if player.attack_cooldown > 0.4 {
            draw_circle_lines(player_screen_pos.x, player_screen_pos.y, player.radius * 2.0, 3.0, YELLOW);
        }
        
        // Draw monsters from all loaded chunks
        for chunk_id in &chunk_manager.loaded_chunks {
            if let Some(chunk) = chunk_manager.chunks.get(chunk_id) {
                // Get sprite renderer for this chunk's biome
                let chunk_sprite_renderer = if let Some(chunk_config) = world_config.chunks.get(chunk_id) {
                    sprite_renderers.get(&chunk_config.biome).unwrap_or(default_sprite_renderer)
                } else {
                    default_sprite_renderer
                };
                
                for monster in &chunk.active_monsters {
                    let monster_screen_pos = monster.position + camera_offset;
                    
                    if !monster.is_dead {
                        let sprite_name = match monster.data.name.as_str() {
                            "Forest Goblin" => "forest_goblin",
                            "Wild Boar" => "wild_boar",
                            "Wolf" => "wolf",
                            "Sand Scorpion" => "sand_scorpion",
                            "Desert Bandit" => "desert_bandit",
                            "Dust Devil" => "dust_devil",
                            "Oasis Guardian" => "oasis_guardian",
                            _ => "forest_goblin",
                        };
                        
                        chunk_sprite_renderer.draw_sprite(
                            sprite_name,
                            "idle",
                            monster_screen_pos,
                            2.0,
                            false
                        );
                        
                        // Health bar
                        let bar_width = 60.0;
                        let bar_height = 6.0;
                        let hp_percent = monster.current_hp as f32 / monster.data.hp as f32;
                        let sprite_height = 16.0 * 2.0;
                        draw_rectangle(
                            monster_screen_pos.x - bar_width/2.0,
                            monster_screen_pos.y + sprite_height/2.0 + 5.0,
                            bar_width,
                            bar_height,
                            DARKGRAY
                        );
                        draw_rectangle(
                            monster_screen_pos.x - bar_width/2.0,
                            monster_screen_pos.y + sprite_height/2.0 + 5.0,
                            bar_width * hp_percent,
                            bar_height,
                            GREEN
                        );
                    }
                }
            }
        }
        
        // Draw damage texts
        for dt in &damage_texts {
            let dt_screen_pos = dt.position + camera_offset;
            draw_text(&dt.text, dt_screen_pos.x - 20.0, dt_screen_pos.y, 24.0, dt.color);
        }
        
        // UI
        draw_text("Seamless RPG World", 10.0, 30.0, 30.0, WHITE);
        draw_text("WASD: Move, SPACE: Attack", 10.0, 60.0, 20.0, WHITE);
        draw_text(&format!("FPS: {}", get_fps()), 10.0, 90.0, 20.0, GREEN);
        
        // Current chunk info
        if let Some(chunk_id) = chunk_manager.get_chunk_at_position(player.position) {
            if let Some(config) = world_config.chunks.get(&chunk_id) {
                draw_text(&format!("Location: {}", config.name), 10.0, 120.0, 20.0, YELLOW);
            }
        }
        
        // World position
        draw_text(&format!("World Pos: ({:.0}, {:.0})", player.position.x, player.position.y), 10.0, 145.0, 16.0, GRAY);
        
        // Loaded chunks info
        draw_text(&format!("Loaded Chunks: {}", chunk_manager.loaded_chunks.len()), 10.0, 165.0, 16.0, GRAY);
        
        // Player stats
        draw_text("=== Player Stats ===", 10.0, 200.0, 20.0, YELLOW);
        draw_text(&format!("HP: {}/{}", player.hp, player.max_hp), 10.0, 230.0, 18.0,
                  if player.hp < 30 { RED } else { WHITE });
        draw_text(&format!("ATK: {}", player.attack), 10.0, 255.0, 18.0, WHITE);
        draw_text(&format!("Gold: {}", gold_collected), 10.0, 280.0, 18.0, GOLD);
        
        // Game over screen
        if game_over {
            let text = "GAME OVER";
            let text_size = 60.0;
            let text_width = measure_text(text, None, text_size as u16, 1.0).width;
            draw_text(text, screen_width()/2.0 - text_width/2.0, screen_height()/2.0, text_size, RED);
            
            let restart_text = "Press R to Restart";
            let restart_size = 30.0;
            let restart_width = measure_text(restart_text, None, restart_size as u16, 1.0).width;
            draw_text(restart_text, screen_width()/2.0 - restart_width/2.0, screen_height()/2.0 + 50.0, restart_size, WHITE);
            
            if is_key_pressed(KeyCode::R) {
                player.hp = player.max_hp;
                player.position = spawn_pos;
                player.facing_left = false;
                player.animation_timer = 0.0;
                gold_collected = 0;
                game_over = false;
            }
        }
        
        next_frame().await
    }
}

pub fn load_monsters(path: &str) -> Vec<Monster> {
    match fs::read_to_string(path) {
        Ok(contents) => {
            match serde_yaml::from_str::<MonsterData>(&contents) {
                Ok(data) => {
                    println!("Monster data loaded from {}", path);
                    data.monsters
                },
                Err(e) => {
                    println!("YAML parsing error: {}", e);
                    vec![]
                }
            }
        },
        Err(e) => {
            println!("File read error: {}", e);
            vec![]
        }
    }
}

pub fn load_map(path: &str) -> Result<MapData, String> {
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
use macroquad::prelude::*;
use serde::{Deserialize, Serialize};
use std::fs;
use std::collections::HashMap;

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

#[derive(Clone)]
struct ActiveMonster {
    data: Monster,
    position: Vec2,
    current_hp: i32,
    is_dead: bool,
    respawn_timer: f32,
}

struct Player {
    position: Vec2,
    hp: i32,
    max_hp: i32,
    attack: i32,
    radius: f32,
    attack_cooldown: f32,
    is_attacking: bool,
}

struct DamageText {
    position: Vec2,
    text: String,
    timer: f32,
    color: Color,
}

// Map structures
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

struct Camera {
    position: Vec2,
}

fn string_to_color(color: &str) -> Color {
    match color {
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

fn tile_to_world(tile_x: usize, tile_y: usize, tile_size: f32) -> Vec2 {
    vec2(
        tile_x as f32 * tile_size + tile_size / 2.0,
        tile_y as f32 * tile_size + tile_size / 2.0
    )
}

fn world_to_tile(world_pos: Vec2, tile_size: f32) -> (usize, usize) {
    (
        (world_pos.x / tile_size) as usize,
        (world_pos.y / tile_size) as usize
    )
}

fn is_position_walkable(pos: Vec2, map_data: &MapData) -> bool {
    let (tile_x, tile_y) = world_to_tile(pos, map_data.map_info.tile_size);
    
    if tile_y >= map_data.layout.len() || tile_x >= map_data.map_info.width {
        return false;
    }
    
    if let Some(row) = map_data.layout.get(tile_y) {
        if let Some(tile_char) = row.chars().nth(tile_x) {
            let tile_key = tile_char.to_string();
            if let Some(tile_type) = map_data.tile_types.get(&tile_key) {
                return tile_type.walkable;
            }
        }
    }
    
    false
}

#[macroquad::main("RPG with Map System")]
async fn main() {
    let monster_templates = load_monsters();
    println!("Loaded monsters: {} types", monster_templates.len());
    
    let map_data = load_map("data/maps/forest_map.yaml");
    println!("Loaded map: {}", map_data.map_info.name);
    
    // Initialize player at spawn point
    let spawn_pos = vec2(
        map_data.map_info.spawn_point.x * map_data.map_info.tile_size,
        map_data.map_info.spawn_point.y * map_data.map_info.tile_size
    );
    
    let mut player = Player {
        position: spawn_pos,
        hp: 100,
        max_hp: 100,
        attack: 15,
        radius: 10.0,
        attack_cooldown: 0.0,
        is_attacking: false,
    };
    
    // Create active monsters from spawn points
    let mut active_monsters: Vec<ActiveMonster> = Vec::new();
    if let Some(spawns) = &map_data.monster_spawns {
        for spawn in spawns {
            if let Some(template) = monster_templates.iter().find(|m| m.name == spawn.monster_type) {
                let world_pos = vec2(
                    spawn.x * map_data.map_info.tile_size,
                    spawn.y * map_data.map_info.tile_size
                );
                
                active_monsters.push(ActiveMonster {
                    data: template.clone(),
                    position: world_pos,
                    current_hp: template.hp,
                    is_dead: false,
                    respawn_timer: 0.0,
                });
            }
        }
    }
    
    let mut damage_texts: Vec<DamageText> = Vec::new();
    let mut gold_collected = 0;
    let mut game_over = false;
    let mut camera = Camera { position: player.position };
    
    loop {
        clear_background(BLACK);
        
        let delta = get_frame_time();
        
        // Update camera to follow player
        camera.position = player.position;
        
        if !game_over {
            // Player movement with collision detection
            let mut move_dir = vec2(0.0, 0.0);
            if is_key_down(KeyCode::W) { move_dir.y -= 1.0; }
            if is_key_down(KeyCode::S) { move_dir.y += 1.0; }
            if is_key_down(KeyCode::A) { move_dir.x -= 1.0; }
            if is_key_down(KeyCode::D) { move_dir.x += 1.0; }
            
            if move_dir.length() > 0.0 {
                let new_pos = player.position + move_dir.normalize() * 200.0 * delta;
                
                // Check all corners of the player for collision
                let corners = [
                    new_pos + vec2(-player.radius, -player.radius),
                    new_pos + vec2(player.radius, -player.radius),
                    new_pos + vec2(-player.radius, player.radius),
                    new_pos + vec2(player.radius, player.radius),
                ];
                
                let mut can_move = true;
                for corner in &corners {
                    if !is_position_walkable(*corner, &map_data) {
                        can_move = false;
                        break;
                    }
                }
                
                if can_move {
                    player.position = new_pos;
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
            
            // Monster AI and combat
            for monster in &mut active_monsters {
                if !monster.is_dead {
                    // Simple AI: move towards player if aggressive
                    if monster.data.behavior.contains(&"aggressive".to_string()) {
                        let dir = (player.position - monster.position).normalize();
                        let new_pos = monster.position + dir * monster.data.speed * 30.0 * delta;
                        
                        // Check if monster can move to new position
                        if is_position_walkable(new_pos, &map_data) {
                            monster.position = new_pos;
                        }
                    }
                    
                    // Check collision with player
                    let monster_radius = 15.0;
                    if check_collision(player.position, player.radius, monster.position, monster_radius) {
                        // Monster attacks player
                        player.hp -= 1;
                        
                        // Player attacks monster if attacking
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
        
        // Draw map
        let tile_size = map_data.map_info.tile_size;
        for (y, row) in map_data.layout.iter().enumerate() {
            for (x, tile_char) in row.chars().enumerate() {
                let tile_key = tile_char.to_string();
                if let Some(tile_type) = map_data.tile_types.get(&tile_key) {
                    let world_pos = vec2(x as f32 * tile_size, y as f32 * tile_size);
                    let screen_pos = world_pos + camera_offset;
                    
                    // Only draw tiles that are on screen
                    if screen_pos.x > -tile_size && screen_pos.x < screen_width() + tile_size &&
                       screen_pos.y > -tile_size && screen_pos.y < screen_height() + tile_size {
                        draw_rectangle(
                            screen_pos.x,
                            screen_pos.y,
                            tile_size,
                            tile_size,
                            string_to_color(&tile_type.color)
                        );
                        
                        // Draw tile borders for better visibility
                        draw_rectangle_lines(
                            screen_pos.x,
                            screen_pos.y,
                            tile_size,
                            tile_size,
                            1.0,
                            Color::new(0.0, 0.0, 0.0, 0.3)
                        );
                    }
                }
            }
        }
        
        // Draw player
        let player_screen_pos = player.position + camera_offset;
        draw_circle(player_screen_pos.x, player_screen_pos.y, player.radius, RED);
        draw_text("Player", player_screen_pos.x - 20.0, player_screen_pos.y - 20.0, 20.0, WHITE);
        
        // Draw attack indicator
        if player.attack_cooldown > 0.4 {
            draw_circle_lines(player_screen_pos.x, player_screen_pos.y, player.radius + 10.0, 3.0, YELLOW);
        }
        
        // Draw monsters
        for monster in &active_monsters {
            let monster_screen_pos = monster.position + camera_offset;
            
            if !monster.is_dead {
                let size = 15.0;
                draw_circle(
                    monster_screen_pos.x, 
                    monster_screen_pos.y, 
                    size, 
                    string_to_color(&monster.data.color)
                );
                
                // Monster info
                draw_text(&monster.data.name, monster_screen_pos.x - 50.0, monster_screen_pos.y - size - 10.0, 16.0, WHITE);
                
                // Health bar
                let bar_width = 60.0;
                let bar_height = 6.0;
                let hp_percent = monster.current_hp as f32 / monster.data.hp as f32;
                draw_rectangle(
                    monster_screen_pos.x - bar_width/2.0, 
                    monster_screen_pos.y + size + 5.0, 
                    bar_width, 
                    bar_height, 
                    DARKGRAY
                );
                draw_rectangle(
                    monster_screen_pos.x - bar_width/2.0, 
                    monster_screen_pos.y + size + 5.0, 
                    bar_width * hp_percent, 
                    bar_height, 
                    GREEN
                );
            } else {
                draw_text(
                    &format!("Respawning in {:.1}s", monster.respawn_timer), 
                    monster_screen_pos.x - 60.0, 
                    monster_screen_pos.y, 
                    16.0, 
                    GRAY
                );
            }
        }
        
        // Draw damage texts
        for dt in &damage_texts {
            let dt_screen_pos = dt.position + camera_offset;
            draw_text(&dt.text, dt_screen_pos.x - 20.0, dt_screen_pos.y, 24.0, dt.color);
        }
        
        // UI (fixed position, not affected by camera)
        draw_text(&format!("{}", map_data.map_info.name), 10.0, 30.0, 30.0, WHITE);
        draw_text("WASD: Move, SPACE: Attack", 10.0, 60.0, 20.0, WHITE);
        draw_text(&format!("FPS: {}", get_fps()), 10.0, 90.0, 20.0, GREEN);
        
        // Player stats
        draw_text("=== Player Stats ===", 10.0, 130.0, 20.0, YELLOW);
        draw_text(&format!("HP: {}/{}", player.hp, player.max_hp), 10.0, 160.0, 18.0, 
                  if player.hp < 30 { RED } else { WHITE });
        draw_text(&format!("ATK: {}", player.attack), 10.0, 185.0, 18.0, WHITE);
        draw_text(&format!("Gold: {}", gold_collected), 10.0, 210.0, 18.0, GOLD);
        
        // Attack cooldown indicator
        if player.attack_cooldown > 0.0 {
            draw_text(&format!("Attack CD: {:.1}s", player.attack_cooldown), 10.0, 235.0, 16.0, GRAY);
        } else {
            draw_text("Attack Ready!", 10.0, 235.0, 16.0, GREEN);
        }
        
        // Minimap
        let minimap_size = 150.0;
        let minimap_x = screen_width() - minimap_size - 10.0;
        let minimap_y = 10.0;
        let minimap_scale = minimap_size / (map_data.map_info.width.max(map_data.map_info.height) as f32 * tile_size);
        
        // Minimap background
        draw_rectangle(minimap_x, minimap_y, minimap_size, minimap_size, Color::new(0.0, 0.0, 0.0, 0.7));
        draw_rectangle_lines(minimap_x, minimap_y, minimap_size, minimap_size, 2.0, WHITE);
        
        // Draw minimap tiles
        for (y, row) in map_data.layout.iter().enumerate() {
            for (x, tile_char) in row.chars().enumerate() {
                let tile_key = tile_char.to_string();
                if let Some(tile_type) = map_data.tile_types.get(&tile_key) {
                    if !tile_type.walkable {
                        let mini_x = minimap_x + (x as f32 * tile_size * minimap_scale);
                        let mini_y = minimap_y + (y as f32 * tile_size * minimap_scale);
                        draw_rectangle(
                            mini_x,
                            mini_y,
                            tile_size * minimap_scale,
                            tile_size * minimap_scale,
                            DARKGRAY
                        );
                    }
                }
            }
        }
        
        // Player on minimap
        let player_mini_x = minimap_x + (player.position.x * minimap_scale);
        let player_mini_y = minimap_y + (player.position.y * minimap_scale);
        draw_circle(player_mini_x, player_mini_y, 3.0, RED);
        
        // Monsters on minimap
        for monster in &active_monsters {
            if !monster.is_dead {
                let monster_mini_x = minimap_x + (monster.position.x * minimap_scale);
                let monster_mini_y = minimap_y + (monster.position.y * minimap_scale);
                draw_circle(monster_mini_x, monster_mini_y, 2.0, YELLOW);
            }
        }
        
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
                // Reset game
                player.hp = player.max_hp;
                player.position = spawn_pos;
                gold_collected = 0;
                game_over = false;
                
                // Reset monsters
                for monster in &mut active_monsters {
                    monster.current_hp = monster.data.hp;
                    monster.is_dead = false;
                    monster.respawn_timer = 0.0;
                }
            }
        }
        
        next_frame().await
    }
}

fn load_monsters() -> Vec<Monster> {
    match fs::read_to_string("data/monsters/forest_monsters_en.yaml") {
        Ok(contents) => {
            match serde_yaml::from_str::<MonsterData>(&contents) {
                Ok(data) => {
                    println!("Monster data loaded successfully!");
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
            vec![Monster {
                name: "Default Goblin".to_string(),
                species: "goblin".to_string(),
                hp: 30,
                attack: 10,
                speed: 1.5,
                color: "green".to_string(),
                behavior: vec!["aggressive".to_string()],
                loot: LootData { gold: 5, item_chance: 0.1 },
            }]
        }
    }
}

fn load_map(path: &str) -> MapData {
    match fs::read_to_string(path) {
        Ok(contents) => {
            match serde_yaml::from_str::<MapData>(&contents) {
                Ok(data) => {
                    println!("Map data loaded successfully!");
                    data
                },
                Err(e) => {
                    println!("Map YAML parsing error: {}", e);
                    create_default_map()
                }
            }
        },
        Err(e) => {
            println!("Map file read error: {}", e);
            create_default_map()
        }
    }
}

fn create_default_map() -> MapData {
    let mut tile_types = HashMap::new();
    tile_types.insert(".".to_string(), TileType {
        name: "grass".to_string(),
        walkable: true,
        color: "green".to_string(),
    });
    tile_types.insert("#".to_string(), TileType {
        name: "wall".to_string(),
        walkable: false,
        color: "gray".to_string(),
    });
    
    MapData {
        map_info: MapInfo {
            name: "Default Map".to_string(),
            width: 20,
            height: 15,
            tile_size: 32.0,
            spawn_point: SpawnPoint { x: 10.0, y: 7.0 },
        },
        tile_types,
        layout: vec![
            "####################".to_string(),
            "#..................#".to_string(),
            "#..................#".to_string(),
            "#..................#".to_string(),
            "#..................#".to_string(),
            "#..................#".to_string(),
            "#..................#".to_string(),
            "#..................#".to_string(),
            "#..................#".to_string(),
            "#..................#".to_string(),
            "#..................#".to_string(),
            "#..................#".to_string(),
            "#..................#".to_string(),
            "#..................#".to_string(),
            "####################".to_string(),
        ],
        monster_spawns: None,
    }
}
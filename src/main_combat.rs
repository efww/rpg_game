use macroquad::prelude::*;
use serde::{Deserialize, Serialize};
use std::fs;

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

fn string_to_color(color: &str) -> Color {
    match color {
        "green" => GREEN,
        "red" => RED,
        "blue" => BLUE,
        "purple" => PURPLE,
        "yellow" => YELLOW,
        "gray" => GRAY,
        _ => WHITE,
    }
}

fn check_collision(pos1: Vec2, radius1: f32, pos2: Vec2, radius2: f32) -> bool {
    (pos1 - pos2).length() < radius1 + radius2
}

#[macroquad::main("RPG with Combat System")]
async fn main() {
    let monster_templates = load_monsters();
    println!("Loaded monsters: {} types", monster_templates.len());
    
    // Initialize player
    let mut player = Player {
        position: vec2(400.0, 300.0),
        hp: 100,
        max_hp: 100,
        attack: 15,
        radius: 20.0,
        attack_cooldown: 0.0,
        is_attacking: false,
    };
    
    // Create active monsters from templates
    let mut active_monsters: Vec<ActiveMonster> = Vec::new();
    for (i, template) in monster_templates.iter().enumerate() {
        let angle = (i as f32) * 2.0 * std::f32::consts::PI / monster_templates.len() as f32;
        let spawn_pos = vec2(
            400.0 + angle.cos() * 200.0,
            300.0 + angle.sin() * 200.0
        );
        
        active_monsters.push(ActiveMonster {
            data: template.clone(),
            position: spawn_pos,
            current_hp: template.hp,
            is_dead: false,
            respawn_timer: 0.0,
        });
    }
    
    let mut damage_texts: Vec<DamageText> = Vec::new();
    let mut gold_collected = 0;
    let mut game_over = false;
    
    loop {
        clear_background(BLACK);
        
        let delta = get_frame_time();
        
        if !game_over {
            // Player movement
            let mut move_dir = vec2(0.0, 0.0);
            if is_key_down(KeyCode::W) { move_dir.y -= 1.0; }
            if is_key_down(KeyCode::S) { move_dir.y += 1.0; }
            if is_key_down(KeyCode::A) { move_dir.x -= 1.0; }
            if is_key_down(KeyCode::D) { move_dir.x += 1.0; }
            
            if move_dir.length() > 0.0 {
                player.position += move_dir.normalize() * 200.0 * delta;
            }
            
            // Attack input
            if is_key_pressed(KeyCode::Space) && player.attack_cooldown <= 0.0 {
                player.is_attacking = true;
                player.attack_cooldown = 0.5; // 0.5 second cooldown
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
                        monster.position += dir * monster.data.speed * 30.0 * delta;
                    }
                    
                    // Check collision with player
                    let monster_radius = 20.0 + (monster.data.hp as f32 / 10.0);
                    if check_collision(player.position, player.radius, monster.position, monster_radius) {
                        // Monster attacks player
                        player.hp -= 1; // Continuous damage while touching
                        
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
                            
                            // Check if monster died
                            if monster.current_hp <= 0 {
                                monster.is_dead = true;
                                monster.respawn_timer = 5.0; // 5 seconds respawn
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
                        // Reset position
                        let angle = rand::gen_range(0.0, std::f32::consts::PI * 2.0);
                        monster.position = vec2(
                            400.0 + angle.cos() * 300.0,
                            300.0 + angle.sin() * 300.0
                        );
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
        
        // Draw player
        draw_circle(player.position.x, player.position.y, player.radius, RED);
        draw_text("Player", player.position.x - 20.0, player.position.y - 30.0, 20.0, WHITE);
        
        // Draw attack indicator
        if player.attack_cooldown > 0.4 {
            draw_circle_lines(player.position.x, player.position.y, player.radius + 10.0, 3.0, YELLOW);
        }
        
        // Draw monsters
        for monster in &active_monsters {
            if !monster.is_dead {
                let size = 20.0 + (monster.data.hp as f32 / 10.0);
                draw_circle(
                    monster.position.x, 
                    monster.position.y, 
                    size, 
                    string_to_color(&monster.data.color)
                );
                
                // Monster info
                draw_text(&monster.data.name, monster.position.x - 50.0, monster.position.y - size - 20.0, 20.0, WHITE);
                
                // Health bar
                let bar_width = 60.0;
                let bar_height = 6.0;
                let hp_percent = monster.current_hp as f32 / monster.data.hp as f32;
                draw_rectangle(
                    monster.position.x - bar_width/2.0, 
                    monster.position.y + size + 10.0, 
                    bar_width, 
                    bar_height, 
                    DARKGRAY
                );
                draw_rectangle(
                    monster.position.x - bar_width/2.0, 
                    monster.position.y + size + 10.0, 
                    bar_width * hp_percent, 
                    bar_height, 
                    GREEN
                );
            } else {
                // Draw respawn timer
                draw_text(
                    &format!("Respawning in {:.1}s", monster.respawn_timer), 
                    monster.position.x - 60.0, 
                    monster.position.y, 
                    16.0, 
                    GRAY
                );
            }
        }
        
        // Draw damage texts
        for dt in &damage_texts {
            draw_text(&dt.text, dt.position.x - 20.0, dt.position.y, 24.0, dt.color);
        }
        
        // UI
        draw_text("RPG Combat System", 10.0, 30.0, 30.0, WHITE);
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
                player.position = vec2(400.0, 300.0);
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
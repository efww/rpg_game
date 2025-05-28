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

// 색상 문자열을 Color로 변환
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

#[macroquad::main("RPG with Data Loading")]
async fn main() {
    // 몬스터 데이터 로드
    let monsters = load_monsters();
    println!("로드된 몬스터: {} 종", monsters.len());
    
    let mut player_pos = vec2(400.0, 300.0);
    let mut selected_monster = 0;
    
    loop {
        clear_background(BLACK);
        
        // 플레이어 이동
        if is_key_down(KeyCode::W) { player_pos.y -= 3.0; }
        if is_key_down(KeyCode::S) { player_pos.y += 3.0; }
        if is_key_down(KeyCode::A) { player_pos.x -= 3.0; }
        if is_key_down(KeyCode::D) { player_pos.x += 3.0; }
        
        // 몬스터 선택 (1-5 키)
        if is_key_pressed(KeyCode::Key1) && monsters.len() > 0 { selected_monster = 0; }
        if is_key_pressed(KeyCode::Key2) && monsters.len() > 1 { selected_monster = 1; }
        if is_key_pressed(KeyCode::Key3) && monsters.len() > 2 { selected_monster = 2; }
        if is_key_pressed(KeyCode::Key4) && monsters.len() > 3 { selected_monster = 3; }
        if is_key_pressed(KeyCode::Key5) && monsters.len() > 4 { selected_monster = 4; }
        
        // 플레이어 그리기
        draw_circle(player_pos.x, player_pos.y, 20.0, RED);
        draw_text("Player", player_pos.x - 20.0, player_pos.y - 30.0, 20.0, WHITE);
        
        // 선택된 몬스터 그리기
        if !monsters.is_empty() {
            let monster = &monsters[selected_monster];
            let monster_pos = vec2(500.0, 300.0);
            
            // 몬스터 크기는 HP에 비례
            let size = 20.0 + (monster.hp as f32 / 10.0);
            draw_circle(
                monster_pos.x, 
                monster_pos.y, 
                size, 
                string_to_color(&monster.color)
            );
            
            // 몬스터 정보
            draw_text(&monster.name, monster_pos.x - 50.0, monster_pos.y - size - 20.0, 20.0, WHITE);
            draw_text(&format!("HP: {}", monster.hp), monster_pos.x - 30.0, monster_pos.y + size + 20.0, 16.0, WHITE);
            draw_text(&format!("ATK: {}", monster.attack), monster_pos.x - 30.0, monster_pos.y + size + 40.0, 16.0, WHITE);
            draw_text(&format!("종족: {}", monster.species), monster_pos.x - 40.0, monster_pos.y + size + 60.0, 16.0, WHITE);
        }
        
        // UI
        draw_text("RPG with YAML Data Loading", 10.0, 30.0, 30.0, WHITE);
        draw_text("WASD: 이동, 1-5: 몬스터 선택", 10.0, 60.0, 20.0, WHITE);
        draw_text(&format!("FPS: {}", get_fps()), 10.0, 90.0, 20.0, GREEN);
        
        // 몬스터 목록
        draw_text("=== 몬스터 목록 ===", 10.0, 150.0, 20.0, YELLOW);
        for (i, monster) in monsters.iter().enumerate() {
            let y_pos = 180.0 + i as f32 * 25.0;
            let color = if i == selected_monster { YELLOW } else { WHITE };
            draw_text(
                &format!("{}: {}", i + 1, monster.name), 
                10.0, 
                y_pos, 
                18.0, 
                color
            );
        }
        
        next_frame().await
    }
}

fn load_monsters() -> Vec<Monster> {
    // YAML 파일 읽기 (영어 버전 사용)
    match fs::read_to_string("data/monsters/forest_monsters_en.yaml") {
        Ok(contents) => {
            match serde_yaml::from_str::<MonsterData>(&contents) {
                Ok(data) => {
                    println!("몬스터 데이터 로드 성공!");
                    data.monsters
                },
                Err(e) => {
                    println!("YAML 파싱 에러: {}", e);
                    vec![]
                }
            }
        },
        Err(e) => {
            println!("파일 읽기 에러: {}", e);
            // 기본 몬스터 반환
            vec![Monster {
                name: "기본 고블린".to_string(),
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
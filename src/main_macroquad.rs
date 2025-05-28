use macroquad::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
struct Monster {
    name: String,
    hp: i32,
    attack: i32,
}

#[macroquad::main("Simple RPG")]
async fn main() {
    // 플레이어 위치
    let mut player_pos = vec2(400.0, 300.0);
    
    // 테스트 몬스터
    let monster = Monster {
        name: "고블린".to_string(),
        hp: 30,
        attack: 10,
    };
    
    loop {
        clear_background(BLACK);
        
        // 플레이어 이동
        if is_key_down(KeyCode::W) { player_pos.y -= 3.0; }
        if is_key_down(KeyCode::S) { player_pos.y += 3.0; }
        if is_key_down(KeyCode::A) { player_pos.x -= 3.0; }
        if is_key_down(KeyCode::D) { player_pos.x += 3.0; }
        
        // 플레이어 그리기 (빨간 원)
        draw_circle(player_pos.x, player_pos.y, 20.0, RED);
        draw_text("Player", player_pos.x - 20.0, player_pos.y - 30.0, 20.0, WHITE);
        
        // 몬스터 그리기 (파란 원)
        draw_circle(500.0, 300.0, 25.0, BLUE);
        draw_text(&monster.name, 480.0, 270.0, 20.0, WHITE);
        draw_text(&format!("HP: {}", monster.hp), 480.0, 340.0, 16.0, WHITE);
        
        // UI
        draw_text("Simple RPG - WASD to move", 10.0, 30.0, 30.0, WHITE);
        draw_text(&format!("FPS: {}", get_fps()), 10.0, 60.0, 20.0, GREEN);
        
        next_frame().await
    }
}
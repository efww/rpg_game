use macroquad::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SpriteData {
    pub sprite_info: SpriteInfo,
    pub color_palette: HashMap<String, String>,
    pub sprites: HashMap<String, CharacterSprite>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SpriteInfo {
    pub width: usize,
    pub height: usize,
    pub description: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CharacterSprite {
    pub name: String,
    pub frames: HashMap<String, Vec<String>>,
}

pub struct SpriteRenderer {
    sprite_data: SpriteData,
    color_cache: HashMap<String, Color>,
}

impl SpriteRenderer {
    pub fn new(sprite_data: SpriteData) -> Self {
        let mut color_cache = HashMap::new();
        
        // Pre-parse colors
        for (key, color_str) in &sprite_data.color_palette {
            let color = parse_color(color_str);
            color_cache.insert(key.clone(), color);
        }
        
        SpriteRenderer {
            sprite_data,
            color_cache,
        }
    }
    
    pub fn draw_sprite(
        &self,
        sprite_name: &str,
        frame_name: &str,
        position: Vec2,
        scale: f32,
        flip_x: bool,
    ) {
        if let Some(sprite) = self.sprite_data.sprites.get(sprite_name) {
            if let Some(frame) = sprite.frames.get(frame_name) {
                let pixel_size = scale;
                let sprite_width = self.sprite_data.sprite_info.width as f32 * pixel_size;
                let sprite_height = self.sprite_data.sprite_info.height as f32 * pixel_size;
                
                // Center the sprite
                let start_x = position.x - sprite_width / 2.0;
                let start_y = position.y - sprite_height / 2.0;
                
                for (y, row) in frame.iter().enumerate() {
                    for (x, ch) in row.chars().enumerate() {
                        let key = ch.to_string();
                        if let Some(color) = self.color_cache.get(&key) {
                            if key != "." {  // Skip transparent pixels
                                let pixel_x = if flip_x {
                                    start_x + (self.sprite_data.sprite_info.width - 1 - x) as f32 * pixel_size
                                } else {
                                    start_x + x as f32 * pixel_size
                                };
                                let pixel_y = start_y + y as f32 * pixel_size;
                                
                                draw_rectangle(pixel_x, pixel_y, pixel_size, pixel_size, *color);
                            }
                        }
                    }
                }
            }
        }
    }
    
    pub fn draw_sprite_outlined(
        &self,
        sprite_name: &str,
        frame_name: &str,
        position: Vec2,
        scale: f32,
        flip_x: bool,
        outline_color: Color,
    ) {
        // Draw outline
        for dx in -1..=1 {
            for dy in -1..=1 {
                if dx != 0 || dy != 0 {
                    let offset_pos = position + vec2(dx as f32, dy as f32);
                    self.draw_sprite_silhouette(sprite_name, frame_name, offset_pos, scale, flip_x, outline_color);
                }
            }
        }
        
        // Draw sprite on top
        self.draw_sprite(sprite_name, frame_name, position, scale, flip_x);
    }
    
    fn draw_sprite_silhouette(
        &self,
        sprite_name: &str,
        frame_name: &str,
        position: Vec2,
        scale: f32,
        flip_x: bool,
        color: Color,
    ) {
        if let Some(sprite) = self.sprite_data.sprites.get(sprite_name) {
            if let Some(frame) = sprite.frames.get(frame_name) {
                let pixel_size = scale;
                let sprite_width = self.sprite_data.sprite_info.width as f32 * pixel_size;
                let sprite_height = self.sprite_data.sprite_info.height as f32 * pixel_size;
                
                let start_x = position.x - sprite_width / 2.0;
                let start_y = position.y - sprite_height / 2.0;
                
                for (y, row) in frame.iter().enumerate() {
                    for (x, ch) in row.chars().enumerate() {
                        if ch != '.' {  // Any non-transparent pixel
                            let pixel_x = if flip_x {
                                start_x + (self.sprite_data.sprite_info.width - 1 - x) as f32 * pixel_size
                            } else {
                                start_x + x as f32 * pixel_size
                            };
                            let pixel_y = start_y + y as f32 * pixel_size;
                            
                            draw_rectangle(pixel_x, pixel_y, pixel_size, pixel_size, color);
                        }
                    }
                }
            }
        }
    }
}

fn parse_color(color_str: &str) -> Color {
    if color_str == "transparent" {
        return Color::new(0.0, 0.0, 0.0, 0.0);
    }
    
    if color_str.starts_with('#') {
        // Parse hex color
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
    
    // Default color names
    match color_str {
        "red" => RED,
        "green" => GREEN,
        "blue" => BLUE,
        "yellow" => YELLOW,
        "purple" => PURPLE,
        "orange" => ORANGE,
        "white" => WHITE,
        "black" => BLACK,
        "gray" => GRAY,
        "darkgray" => DARKGRAY,
        "lightgray" => LIGHTGRAY,
        _ => WHITE,
    }
}

pub fn load_sprites(path: &str) -> Result<SpriteData, String> {
    match std::fs::read_to_string(path) {
        Ok(contents) => {
            match serde_json::from_str::<SpriteData>(&contents) {
                Ok(data) => Ok(data),
                Err(e) => Err(format!("JSON parsing error: {}", e)),
            }
        },
        Err(e) => Err(format!("File read error: {}", e)),
    }
}
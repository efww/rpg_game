use macroquad::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::{MapData, Monster, ActiveMonster, load_map};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct WorldConfig {
    pub world_info: WorldInfo,
    pub chunk_layout: Vec<Vec<String>>,
    pub chunks: HashMap<String, ChunkConfig>,
    pub spawn_chunk: String,
    pub spawn_position: SpawnPosition,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct WorldInfo {
    pub name: String,
    pub chunk_size: usize,
    pub tile_size: f32,
    pub chunks_x: usize,
    pub chunks_y: usize,
    pub view_distance: usize,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ChunkConfig {
    pub world_x: usize,
    pub world_y: usize,
    pub biome: String,
    pub map_file: String,
    pub name: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SpawnPosition {
    pub x: f32,
    pub y: f32,
}

pub struct Chunk {
    pub id: String,
    pub world_x: usize,
    pub world_y: usize,
    pub map_data: MapData,
    pub active_monsters: Vec<ActiveMonster>,
    pub is_loaded: bool,
}

pub struct ChunkManager {
    pub world_config: WorldConfig,
    pub chunks: HashMap<String, Chunk>,
    pub loaded_chunks: Vec<String>,
    pub current_chunk: String,
    pub chunk_pixel_size: f32,
}

impl ChunkManager {
    pub fn new(world_config: WorldConfig) -> Self {
        let chunk_pixel_size = world_config.world_info.chunk_size as f32 * world_config.world_info.tile_size;
        
        ChunkManager {
            world_config,
            chunks: HashMap::new(),
            loaded_chunks: Vec::new(),
            current_chunk: String::new(),
            chunk_pixel_size,
        }
    }
    
    pub fn initialize(&mut self, monster_templates: &[Monster]) {
        // Load spawn chunk
        let spawn_chunk = self.world_config.spawn_chunk.clone();
        self.load_chunk(&spawn_chunk, monster_templates);
        self.current_chunk = spawn_chunk;
    }
    
    pub fn world_to_chunk_coords(&self, world_pos: Vec2) -> (usize, usize) {
        let chunk_x = (world_pos.x / self.chunk_pixel_size) as usize;
        let chunk_y = (world_pos.y / self.chunk_pixel_size) as usize;
        (chunk_x, chunk_y)
    }
    
    pub fn chunk_to_world_coords(&self, chunk_x: usize, chunk_y: usize) -> Vec2 {
        vec2(
            chunk_x as f32 * self.chunk_pixel_size,
            chunk_y as f32 * self.chunk_pixel_size
        )
    }
    
    pub fn get_chunk_at_position(&self, world_pos: Vec2) -> Option<String> {
        let (chunk_x, chunk_y) = self.world_to_chunk_coords(world_pos);
        
        if chunk_y < self.world_config.chunk_layout.len() &&
           chunk_x < self.world_config.chunk_layout[chunk_y].len() {
            Some(self.world_config.chunk_layout[chunk_y][chunk_x].clone())
        } else {
            None
        }
    }
    
    pub fn update_loaded_chunks(&mut self, player_pos: Vec2, monster_templates: &[Monster]) {
        let current_chunk_id = self.get_chunk_at_position(player_pos);
        
        if let Some(chunk_id) = current_chunk_id {
            if chunk_id != self.current_chunk {
                self.current_chunk = chunk_id.clone();
                println!("Entered chunk: {}", chunk_id);
            }
            
            // Get chunks that should be loaded
            let chunks_to_load = self.get_nearby_chunks(&chunk_id);
            
            // Unload chunks that are too far
            let mut chunks_to_unload = Vec::new();
            for loaded_id in &self.loaded_chunks {
                if !chunks_to_load.contains(loaded_id) {
                    chunks_to_unload.push(loaded_id.clone());
                }
            }
            
            for chunk_id in chunks_to_unload {
                self.unload_chunk(&chunk_id);
            }
            
            // Load new chunks
            for chunk_id in chunks_to_load {
                if !self.loaded_chunks.contains(&chunk_id) {
                    self.load_chunk(&chunk_id, monster_templates);
                }
            }
        }
    }
    
    fn get_nearby_chunks(&self, center_chunk_id: &str) -> Vec<String> {
        let mut nearby = vec![center_chunk_id.to_string()];
        
        if let Some(config) = self.world_config.chunks.get(center_chunk_id) {
            let view_dist = self.world_config.world_info.view_distance as i32;
            
            for dy in -view_dist..=view_dist {
                for dx in -view_dist..=view_dist {
                    let nx = config.world_x as i32 + dx;
                    let ny = config.world_y as i32 + dy;
                    
                    if nx >= 0 && ny >= 0 &&
                       ny < self.world_config.chunk_layout.len() as i32 &&
                       nx < self.world_config.chunk_layout[ny as usize].len() as i32 {
                        let chunk_id = &self.world_config.chunk_layout[ny as usize][nx as usize];
                        if !nearby.contains(chunk_id) {
                            nearby.push(chunk_id.clone());
                        }
                    }
                }
            }
        }
        
        nearby
    }
    
    fn load_chunk(&mut self, chunk_id: &str, monster_templates: &[Monster]) {
        if let Some(config) = self.world_config.chunks.get(chunk_id) {
            match load_map(&config.map_file) {
                Ok(map_data) => {
                    let mut chunk = Chunk {
                        id: chunk_id.to_string(),
                        world_x: config.world_x,
                        world_y: config.world_y,
                        map_data,
                        active_monsters: Vec::new(),
                        is_loaded: true,
                    };
                    
                    // Create monsters for this chunk
                    if let Some(spawns) = &chunk.map_data.monster_spawns {
                        let world_offset = self.chunk_to_world_coords(config.world_x, config.world_y);
                        
                        for spawn in spawns {
                            if let Some(template) = monster_templates.iter().find(|m| m.name == spawn.monster_type) {
                                let world_pos = world_offset + vec2(
                                    spawn.x * chunk.map_data.map_info.tile_size,
                                    spawn.y * chunk.map_data.map_info.tile_size
                                );
                                
                                chunk.active_monsters.push(ActiveMonster {
                                    data: template.clone(),
                                    position: world_pos,
                                    current_hp: template.hp,
                                    is_dead: false,
                                    respawn_timer: 0.0,
                                });
                            }
                        }
                    }
                    
                    self.chunks.insert(chunk_id.to_string(), chunk);
                    self.loaded_chunks.push(chunk_id.to_string());
                    println!("Loaded chunk: {}", chunk_id);
                },
                Err(e) => {
                    println!("Failed to load chunk {}: {}", chunk_id, e);
                }
            }
        }
    }
    
    fn unload_chunk(&mut self, chunk_id: &str) {
        if self.chunks.remove(chunk_id).is_some() {
            self.loaded_chunks.retain(|id| id != chunk_id);
            println!("Unloaded chunk: {}", chunk_id);
        }
    }
    
    pub fn is_position_walkable(&self, world_pos: Vec2) -> bool {
        let chunk_id = match self.get_chunk_at_position(world_pos) {
            Some(id) => id,
            None => return false,
        };
        
        let chunk = match self.chunks.get(&chunk_id) {
            Some(c) => c,
            None => return false,
        };
        
        // Convert world position to local chunk position
        let world_offset = self.chunk_to_world_coords(chunk.world_x, chunk.world_y);
        let local_pos = world_pos - world_offset;
        
        // Convert to tile coordinates
        let tile_size = chunk.map_data.map_info.tile_size;
        let tile_x = (local_pos.x / tile_size) as usize;
        let tile_y = (local_pos.y / tile_size) as usize;
        
        if tile_y >= chunk.map_data.layout.len() || tile_x >= chunk.map_data.map_info.width {
            return false;
        }
        
        if let Some(row) = chunk.map_data.layout.get(tile_y) {
            if let Some(tile_char) = row.chars().nth(tile_x) {
                let tile_key = tile_char.to_string();
                if let Some(tile_type) = chunk.map_data.tile_types.get(&tile_key) {
                    return tile_type.walkable;
                }
            }
        }
        
        false
    }
    
    pub fn draw_chunks(&self, camera_offset: Vec2, _sprite_renderer: &crate::sprite_system::SpriteRenderer) {
        // Draw all loaded chunks
        for chunk_id in &self.loaded_chunks {
            if let Some(chunk) = self.chunks.get(chunk_id) {
                self.draw_chunk(chunk, camera_offset);
            }
        }
    }
    
    fn draw_chunk(&self, chunk: &Chunk, camera_offset: Vec2) {
        let world_offset = self.chunk_to_world_coords(chunk.world_x, chunk.world_y);
        let tile_size = chunk.map_data.map_info.tile_size;
        
        for (y, row) in chunk.map_data.layout.iter().enumerate() {
            for (x, tile_char) in row.chars().enumerate() {
                let tile_key = tile_char.to_string();
                if let Some(tile_type) = chunk.map_data.tile_types.get(&tile_key) {
                    let world_pos = world_offset + vec2(x as f32 * tile_size, y as f32 * tile_size);
                    let screen_pos = world_pos + camera_offset;
                    
                    // Only draw tiles that are on screen
                    if screen_pos.x > -tile_size && screen_pos.x < screen_width() + tile_size &&
                       screen_pos.y > -tile_size && screen_pos.y < screen_height() + tile_size {
                        draw_rectangle(
                            screen_pos.x,
                            screen_pos.y,
                            tile_size,
                            tile_size,
                            crate::string_to_color(&tile_type.color)
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
    }
    
    pub fn get_loaded_chunk_ids(&self) -> Vec<String> {
        self.loaded_chunks.clone()
    }
    
    pub fn get_chunk_mut(&mut self, chunk_id: &str) -> Option<&mut Chunk> {
        self.chunks.get_mut(chunk_id)
    }
}

pub fn load_world_config(path: &str) -> Result<WorldConfig, String> {
    match std::fs::read_to_string(path) {
        Ok(contents) => {
            match serde_json::from_str::<WorldConfig>(&contents) {
                Ok(config) => Ok(config),
                Err(e) => Err(format!("JSON parsing error: {}", e)),
            }
        },
        Err(e) => Err(format!("File read error: {}", e)),
    }
}
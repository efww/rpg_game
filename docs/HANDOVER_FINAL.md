# RPG Game Development Handover Document
**Date**: May 27, 2025  
**Project**: Seamless Open World RPG with Data-Driven Design

## 🎮 Project Overview

This is a modular, data-driven RPG game built with Rust and Macroquad. The game features a seamless 3x3 chunk world system, JRPG-style pixel art sprites, and a complete combat system. All game content (maps, monsters, items) is defined in YAML/JSON files, implementing the "Data is the Game" philosophy.

## 🚀 Quick Start

```bash
# Clone the repository
git clone https://github.com/efww/rpg_game.git
cd rpg_game

# Run the seamless world version
cargo run --bin rpg_seamless

# Run the single-map version
cargo run --bin rpg_game
```

## 🏗️ Architecture

### Core Systems

1. **Chunk System** (`src/chunk_system.rs`)
   - 3x3 world grid with dynamic loading
   - View distance-based chunk streaming
   - Seamless transitions between biomes

2. **Sprite System** (`src/sprite_system.rs`)
   - JSON-defined pixel art sprites
   - 16x16 JRPG-style characters
   - Support for animations and sprite flipping

3. **Biome System** (`src/biome_system.rs`)
   - Configuration-based biome management
   - Different monster sets per biome
   - Extensible for new environments

### Game Features

- **Combat System**: Real-time combat with attack cooldowns
- **Movement**: 8-directional movement with wall sliding
- **Monster AI**: Aggressive monsters that chase players
- **Respawn System**: Defeated monsters respawn after 5 seconds
- **Gold Collection**: Loot system for defeated enemies

## 📁 Project Structure

```
rpg_game/
├── src/
│   ├── main.rs                    # Single-map version
│   ├── main_seamless_fixed.rs     # Seamless world version
│   ├── chunk_system.rs            # World chunk management
│   ├── sprite_system.rs           # Sprite rendering
│   └── biome_system.rs            # Biome configuration
├── data/
│   ├── world_config.json          # World layout configuration
│   ├── game_config.json           # Game settings
│   ├── maps/
│   │   ├── forest_map.yaml        # Central forest map
│   │   ├── desert_town.yaml       # Desert biome map
│   │   └── chunks/                # Additional forest chunks
│   ├── monsters/
│   │   ├── forest_monsters_en.yaml # Forest creatures
│   │   └── desert_monsters.yaml    # Desert creatures
│   └── sprites/
│       ├── character_sprites.json  # Player & forest sprites
│       └── desert_sprites.json     # Desert biome sprites
└── Cargo.toml                     # Rust dependencies
```

## 🗺️ World Layout

```
[Forest] [Forest] [Forest]
[Desert] [Forest] [Forest]  <- Player spawns in center
[Forest] [Forest] [Forest]
```

- **Central Chunk (1,1)**: Starting forest area
- **Western Chunk (1,0)**: Desert town with buildings
- **Other Chunks**: Additional forest areas

## 🎨 Content Creation

### Adding New Maps

1. Create a YAML file in `data/maps/`:
```yaml
map_info:
  name: "New Area"
  width: 30
  height: 20
  tile_size: 32
  spawn_point:
    x: 15
    y: 10

tile_types:
  ".": 
    name: "grass"
    walkable: true
    color: "#00FF00"
  "#":
    name: "wall"
    walkable: false
    color: "#808080"

layout:
  - "##############################"
  - "#............................#"
  # ... more rows
```

2. Add to `world_config.json` for seamless integration

### Adding New Monsters

1. Create entry in monster YAML:
```yaml
monsters:
  - name: "New Monster"
    species: "creature_type"
    hp: 50
    attack: 15
    speed: 1.5
    color: "purple"
    behavior:
      - "aggressive"
    loot:
      gold: 20
      item_chance: 0.3
```

2. Create sprite in JSON format (16x16 pixel art)

### Creating Sprites

Sprites use character-based pixel art in JSON:
```json
"new_monster": {
  "name": "New Monster",
  "frames": {
    "idle": [
      "....RRRR....",
      "...RRRRRR...",
      // 16 rows of 16 characters each
    ]
  }
}
```

## 🔧 Key Technologies

- **Language**: Rust
- **Game Engine**: Macroquad (lightweight, cross-platform)
- **Data Format**: YAML (maps, monsters), JSON (sprites, config)
- **Graphics**: Programmatically generated pixel art

## 🎯 Next Development Steps

### Immediate (1-2 days)
- [ ] Fix any remaining Korean encoding issues
- [ ] Add more monster varieties
- [ ] Implement basic items and inventory

### Short Term (1 week)
- [ ] Quest system with NPC dialogues
- [ ] Save/Load game functionality
- [ ] Sound effects and music
- [ ] Weather system per biome

### Long Term (1 month)
- [ ] Multiplayer support
- [ ] Procedural dungeon generation
- [ ] Skill trees and character progression
- [ ] Boss battles with special mechanics

## 🐛 Known Issues

1. **Korean Text**: Some systems may show Korean text as "???" - use English data files
2. **Performance**: Large numbers of monsters (>50) may impact FPS
3. **Collision**: Wall sliding works but may feel "sticky" in tight corners

## 💡 Design Philosophy

This project follows the "Data is the Game" principle:
- Game logic is minimal and reusable
- Content is entirely data-driven
- New features = new data files, not new code
- AI can generate compatible content

## 📞 Support

For questions or contributions:
- Repository: https://github.com/efww/rpg_game.git
- Create issues for bugs or feature requests

---

*This project demonstrates how modern game development can leverage data-driven design and AI content generation to create scalable, maintainable games with minimal code.*
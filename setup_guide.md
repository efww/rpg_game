# PRD 기반 RPG 게임 개발 시작 가이드

## 1. 프로젝트 초기 설정

### 엔진 선택
- **Macroquad**: 빠른 프로토타이핑, 간단한 2D/3D
- **Bevy**: 본격적인 게임, ECS 구조, 확장성

### 추천: Bevy로 시작
```bash
cargo new rpg_game
cd rpg_game
```

Cargo.toml:
```toml
[dependencies]
bevy = "0.13"
serde = { version = "1.0", features = ["derive"] }
serde_yaml = "0.9"
serde_json = "1.0"
```

## 2. 기본 폴더 구조

```
rpg_game/
├── src/
│   ├── main.rs
│   ├── components/   # ECS 컴포넌트
│   ├── systems/      # 게임 시스템
│   └── data/         # 데이터 로더
├── assets/
│   ├── models/       # 3D 모델 (나중에)
│   └── textures/     # 텍스처 (나중에)
└── data/
    ├── templates/    # 콘텐츠 템플릿
    ├── monsters/     # AI 생성 몬스터
    ├── items/        # AI 생성 아이템
    └── maps/         # AI 생성 맵
```

## 3. 첫 번째 코드: 기본 구조

src/main.rs:
```rust
use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .add_systems(Update, movement_system)
        .run();
}

fn setup(mut commands: Commands) {
    // 카메라
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(0.0, 5.0, 10.0)
            .looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });
    
    // 플레이어 (일단 큐브로)
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(Cube::new(1.0)),
            material: materials.add(Color::rgb(0.8, 0.2, 0.2)),
            ..default()
        },
        Player,
        Health { current: 100, max: 100 },
    ));
}
```

## 4. 콘텐츠 템플릿 예시

data/templates/monster_template.yaml:
```yaml
# 몬스터 기본 템플릿
template_id: base_monster
fields:
  name: 
    type: string
    required: true
  species:
    type: string
    options: [goblin, orc, wolf, dragon]
  stats:
    hp: 
      type: integer
      min: 10
      max: 1000
    attack:
      type: integer
      min: 5
      max: 100
  behaviors:
    type: array
    options: [patrol, aggressive, defensive, flee]
```

## 5. AI 프롬프트 예시

"위 monster_template.yaml을 기반으로 숲 지역에 서식하는 몬스터 5종을 생성해줘. 
각 몬스터는 고유한 특성과 행동 패턴을 가져야 해."

## 다음 단계

1. 기본 ECS 컴포넌트 정의
2. YAML 로더 구현
3. AI로 첫 콘텐츠 생성
4. 간단한 전투 시스템
5. 맵 시스템 추가
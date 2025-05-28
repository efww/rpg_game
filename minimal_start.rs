// 가장 간단한 시작 - main.rs에 복사
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

// 1. 컴포넌트 정의
#[derive(Component)]
struct Player;

#[derive(Component, Serialize, Deserialize)]
struct Health {
    current: i32,
    max: i32,
}

#[derive(Component, Serialize, Deserialize)]
struct Monster {
    name: String,
    attack: i32,
}

// 2. 메인 함수
fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .add_systems(Update, (movement_system, load_test))
        .run();
}

// 3. 초기 설정
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // 카메라
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(0.0, 5.0, 10.0)
            .looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });

    // 플레이어 (빨간 큐브)
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(Cuboid::new(1.0, 1.0, 1.0)),
            material: materials.add(Color::rgb(0.8, 0.2, 0.2)),
            ..default()
        },
        Player,
        Health { current: 100, max: 100 },
    ));

    // 몬스터 (파란 큐브)
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(Cuboid::new(1.0, 1.0, 1.0)),
            material: materials.add(Color::rgb(0.2, 0.2, 0.8)),
            transform: Transform::from_xyz(3.0, 0.5, 0.0),
            ..default()
        },
        Monster {
            name: "고블린".to_string(),
            attack: 10,
        },
        Health { current: 30, max: 30 },
    ));

    // 빛
    commands.spawn(DirectionalLightBundle {
        transform: Transform::from_rotation(Quat::from_euler(
            EulerRot::XYZ, -45.0_f32.to_radians(), -45.0_f32.to_radians(), 0.0
        )),
        ..default()
    });
}

// 4. 이동 시스템
fn movement_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut query: Query<&mut Transform, With<Player>>,
) {
    for mut transform in &mut query {
        let speed = 3.0;
        let delta = time.delta_seconds();

        if keyboard.pressed(KeyCode::KeyW) {
            transform.translation.z -= speed * delta;
        }
        if keyboard.pressed(KeyCode::KeyS) {
            transform.translation.z += speed * delta;
        }
        if keyboard.pressed(KeyCode::KeyA) {
            transform.translation.x -= speed * delta;
        }
        if keyboard.pressed(KeyCode::KeyD) {
            transform.translation.x += speed * delta;
        }
    }
}

// 5. 데이터 로드 테스트
fn load_test() {
    // 나중에 YAML 로드 추가
}
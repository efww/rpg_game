@echo off
echo ==========================================
echo    PRD 기반 RPG 게임 개발 시작
echo ==========================================
echo.

echo [1/5] 프로젝트 생성 중...
cargo new rpg_game --bin
cd rpg_game

echo.
echo [2/5] 폴더 구조 생성 중...
mkdir src\components
mkdir src\systems  
mkdir src\data
mkdir assets\models
mkdir assets\textures
mkdir data\templates
mkdir data\monsters
mkdir data\items
mkdir data\maps

echo.
echo [3/5] Cargo.toml 생성 중...
echo [package] > Cargo.toml
echo name = "rpg_game" >> Cargo.toml
echo version = "0.1.0" >> Cargo.toml
echo edition = "2021" >> Cargo.toml
echo. >> Cargo.toml
echo [dependencies] >> Cargo.toml
echo bevy = "0.13" >> Cargo.toml
echo serde = { version = "1.0", features = ["derive"] } >> Cargo.toml
echo serde_yaml = "0.9" >> Cargo.toml
echo serde_json = "1.0" >> Cargo.toml

echo.
echo [4/5] 기본 템플릿 생성 중...
echo # Monster Template > data\templates\monster_template.yaml
echo template_id: base_monster >> data\templates\monster_template.yaml
echo fields: >> data\templates\monster_template.yaml
echo   name: string >> data\templates\monster_template.yaml
echo   hp: [10, 1000] >> data\templates\monster_template.yaml
echo   attack: [5, 100] >> data\templates\monster_template.yaml

echo.
echo [5/5] 완료!
echo.
echo ==========================================
echo    다음 단계:
echo    1. cd rpg_game
echo    2. code . (VS Code 열기)
echo    3. cargo run (실행)
echo ==========================================
pause
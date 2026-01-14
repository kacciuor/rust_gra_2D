use bevy::prelude::*;
use rand::Rng;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .init_state::<GameState>()
        .init_resource::<Score>()
        .init_resource::<Difficulty>()
        .init_resource::<SpawnerDistance>()
        .add_systems(Startup, setup)
        .add_systems(OnEnter(GameState::Menu), spawn_menu)
        .add_systems(Update, menu_logic.run_if(in_state(GameState::Menu)))
        .add_systems(OnExit(GameState::Menu), despawn_screen::<MenuScreen>)
        .add_systems(Update, (
            player_movement,
            spawn_obstacles,
            spawn_coins,
            move_everything,
            increase_difficulty,
            check_collisions,
            collect_coins,
            update_game_ui,
        ).run_if(in_state(GameState::Playing)))
        .add_systems(OnEnter(GameState::GameOver), spawn_game_over)
        .add_systems(Update, game_over_logic.run_if(in_state(GameState::GameOver)))
        .add_systems(OnExit(GameState::GameOver), (despawn_screen::<GameOverScreen>, reset_game_data))
        .run();
}

#[derive(States, Debug, Clone, PartialEq, Eq, Hash, Default)]
enum GameState {
    #[default]
    Menu,
    Playing,
    GameOver,
}

#[derive(Component)]
struct Player { velocity: f32, health: i32, invulnerable_timer: f32 }
#[derive(Component)]
struct Obstacle;
#[derive(Component)]
struct Coin;
#[derive(Component)]
struct Movable;
#[derive(Resource, Default)]
struct Score(u32);
#[derive(Resource, Default)]
struct SpawnerDistance { obstacle_dist: f32, coin_dist: f32 }
#[derive(Component)]
struct HudText;
#[derive(Component)]
struct MenuScreen;
#[derive(Component)]
struct GameOverScreen;

#[derive(Resource)]
struct Difficulty { speed: f32, timer: Timer }

impl Default for Difficulty {
    fn default() -> Self {
        Difficulty {
            speed: 400.0,
            timer: Timer::from_seconds(5.0, TimerMode::Repeating),
        }
    }
}

const GROUND_Y: f32 = -100.0;
const JUMP_FORCE: f32 = 700.0;
const GRAVITY: f32 = -2000.0;

fn setup(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>, mut materials: ResMut<Assets<ColorMaterial>>) {
    commands.spawn(Camera2d);

    commands.spawn((
        Mesh2d(meshes.add(Rectangle::new(50.0, 50.0))),
        MeshMaterial2d(materials.add(Color::from(LinearRgba::GREEN))),
        Transform::from_xyz(-200.0, GROUND_Y, 0.0),
        Player { velocity: 0.0, health: 3, invulnerable_timer: 0.0 },
    ));

    commands.spawn(Node {
        position_type: PositionType::Absolute,
        top: Val::Px(20.0),
        left: Val::Px(20.0),
        ..default()
    }).with_children(|parent| {
        parent.spawn((
            Text::new(""),
            TextFont { font_size: 25.0, ..default() },
            HudText,
        ));
    });
}

fn spawn_menu(mut commands: Commands) {
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            flex_direction: FlexDirection::Column,
            ..default()
        },
        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.9)), // Poprawione: BackgroundColor poza Node
        MenuScreen,
    )).with_children(|parent| {
        parent.spawn((
            Text::new("ENDLESS RUNNER 2D"), 
            TextFont { font_size: 80.0, ..default() },
            TextColor(Color::WHITE),
        ));
        parent.spawn((
            Text::new("PRESS SPACE TO PLAY"), 
            TextFont { font_size: 30.0, ..default() },
            TextColor(Color::from(bevy::color::palettes::css::GOLD)),
        ));
    });
}

fn spawn_game_over(mut commands: Commands, score: Res<Score>) {
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            flex_direction: FlexDirection::Column,
            ..default()
        },
        BackgroundColor(Color::srgba(0.2, 0.0, 0.0, 0.9)), 
        GameOverScreen,
    )).with_children(|parent| {
        parent.spawn((
            Text::new("GAME OVER"), 
            TextFont { font_size: 80.0, ..default() },
            TextColor(Color::from(bevy::color::palettes::css::RED)),
        ));
        parent.spawn((
            Text::new(format!("Final Score: {}", score.0)), 
            TextFont { font_size: 40.0, ..default() },
            TextColor(Color::WHITE),
        ));
        parent.spawn((
            Text::new("PRESS SPACE TO RESTART"), 
            TextFont { font_size: 30.0, ..default() },
            TextColor(Color::WHITE),
        ));
    });
}

fn menu_logic(keys: Res<ButtonInput<KeyCode>>, mut next: ResMut<NextState<GameState>>) {
    if keys.just_pressed(KeyCode::Space) { next.set(GameState::Playing); }
}

fn game_over_logic(keys: Res<ButtonInput<KeyCode>>, mut next: ResMut<NextState<GameState>>) {
    if keys.just_pressed(KeyCode::Space) { next.set(GameState::Playing); }
}

fn player_movement(keyboard: Res<ButtonInput<KeyCode>>, time: Res<Time>, mut query: Query<(&mut Transform, &mut Player)>) {
    let delta = time.delta_secs();
    for (mut transform, mut player) in &mut query {
        let on_ground = transform.translation.y <= GROUND_Y + 0.1;
        if keyboard.just_pressed(KeyCode::Space) && on_ground { player.velocity = JUMP_FORCE; }
        if !on_ground { player.velocity += GRAVITY * delta; }
        transform.translation.y = (transform.translation.y + player.velocity * delta).max(GROUND_Y);
        if transform.translation.y == GROUND_Y { player.velocity = 0.0; }
    }
}

fn spawn_obstacles(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>, mut materials: ResMut<Assets<ColorMaterial>>, time: Res<Time>, diff: Res<Difficulty>, mut spawner: ResMut<SpawnerDistance>) {
    spawner.obstacle_dist += diff.speed * time.delta_secs();
    if spawner.obstacle_dist >= 800.0 {
        spawner.obstacle_dist = 0.0;
        commands.spawn((
            Mesh2d(meshes.add(Rectangle::new(40.0, 80.0))),
            MeshMaterial2d(materials.add(Color::from(bevy::color::palettes::css::RED))),
            Transform::from_xyz(1000.0, GROUND_Y + 15.0, 0.0),
            Obstacle, Movable,
        ));
    }
}

fn spawn_coins(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>, mut materials: ResMut<Assets<ColorMaterial>>, time: Res<Time>, diff: Res<Difficulty>, mut spawner: ResMut<SpawnerDistance>) {
    spawner.coin_dist += diff.speed * time.delta_secs();
    if spawner.coin_dist >= 1200.0 {
        spawner.coin_dist = 0.0;
        let mut rng = rand::rng();
        let random_y = GROUND_Y + rng.random_range(50.0..180.0);
        commands.spawn((
            Mesh2d(meshes.add(Rectangle::new(25.0, 25.0))),
            MeshMaterial2d(materials.add(Color::from(bevy::color::palettes::css::YELLOW))),
            Transform::from_xyz(1000.0, random_y, 0.1),
            Coin, Movable,
        ));
    }
}

fn move_everything(mut commands: Commands, diff: Res<Difficulty>, mut query: Query<(Entity, &mut Transform), With<Movable>>, time: Res<Time>) {
    for (entity, mut transform) in &mut query {
        transform.translation.x -= diff.speed * time.delta_secs();
        if transform.translation.x < -1000.0 { commands.entity(entity).despawn(); }
    }
}

fn check_collisions(
    mut next: ResMut<NextState<GameState>>,
    time: Res<Time>,
    mut player_q: Query<(&mut Player, &Transform, &MeshMaterial2d<ColorMaterial>)>, 
    obs_q: Query<&Transform, (With<Obstacle>, Without<Player>)>, 
    mut materials: ResMut<Assets<ColorMaterial>>
) {
    let delta = time.delta_secs();
    for (mut player, p_tf, mat_handle) in &mut player_q {
        if player.invulnerable_timer > 0.0 {
            player.invulnerable_timer -= delta;
            if let Some(mat) = materials.get_mut(&mat_handle.0) { mat.color = Color::Srgba(bevy::color::palettes::css::YELLOW); }
            continue;
        } else if let Some(mat) = materials.get_mut(&mat_handle.0) {
            mat.color = Color::from(LinearRgba::GREEN);
        }

        for o_tf in &obs_q {
            if (p_tf.translation.x - o_tf.translation.x).abs() < 45.0 && (p_tf.translation.y - o_tf.translation.y).abs() < 65.0 {
                player.health -= 1;
                if player.health <= 0 { next.set(GameState::GameOver); }
                else { player.invulnerable_timer = 1.0; }
            }
        }
    }
}

fn collect_coins(mut commands: Commands, player_q: Query<&Transform, With<Player>>, coin_q: Query<(Entity, &Transform), (With<Coin>, Without<Player>)>, mut score: ResMut<Score>) {
    for p_tf in &player_q {
        for (c_ent, c_tf) in &coin_q {
            if (p_tf.translation.x - c_tf.translation.x).abs() < 35.0 && (p_tf.translation.y - c_tf.translation.y).abs() < 35.0 {
                score.0 += 1;
                commands.entity(c_ent).despawn();
            }
        }
    }
}

fn increase_difficulty(time: Res<Time>, mut diff: ResMut<Difficulty>) {
    diff.timer.tick(time.delta());
    if diff.timer.just_finished() && diff.speed < 1200.0 { diff.speed += 50.0; }
}

fn update_game_ui(score: Res<Score>, diff: Res<Difficulty>, player_q: Query<&Player>, mut ui_q: Query<&mut Text, With<HudText>>) {
    for player in &player_q {
        for mut text in &mut ui_q {
            text.0 = format!("HP: {}\nScore: {}\nSpeed: {:.0}", player.health, score.0, diff.speed);
        }
    }
}

fn despawn_screen<T: Component>(mut commands: Commands, query: Query<Entity, With<T>>) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}

fn reset_game_data(
    mut score: ResMut<Score>,
    mut diff: ResMut<Difficulty>,
    mut spawner: ResMut<SpawnerDistance>,
    mut player_q: Query<(&mut Transform, &mut Player)>,
    movable_q: Query<Entity, With<Movable>>,
    mut commands: Commands,
) {
    score.0 = 0;
    diff.speed = 400.0;
    spawner.obstacle_dist = 0.0;
    spawner.coin_dist = 0.0;
    for (mut tf, mut p) in &mut player_q {
        p.health = 3; p.velocity = 0.0; tf.translation.y = GROUND_Y;
    }
    for entity in &movable_q {
        commands.entity(entity).despawn();
    }
}
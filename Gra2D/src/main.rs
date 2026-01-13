use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .init_resource::<Score>()
        .add_systems(Startup, setup)
        .add_systems(Update, (
            player_movement,
            spawn_obstacles, 
            move_everything, 
            check_collisions, 
            update_health_ui,
            spawn_coins,      
            collect_coins,    
            update_ui
        ))
        .run();
}

#[derive(Component)]
struct Player { 
    velocity: f32,
    health: i32,
    invulnerable_timer: f32, 
}

#[derive(Component)]
struct Obstacle;

#[derive(Component)]
struct HealthText;

#[derive(Component)]
struct Coin;

#[derive(Resource)]
struct Score(u32); 

impl Default for Score {
    fn default() -> Self {
        Score(0)
    }
}

#[derive(Component)]
struct ScoreText; 

#[derive(Component)]
struct Movable;

const GROUND_Y: f32 = -100.0;
const JUMP_FORCE: f32 = 700.0;
const GRAVITY: f32 = -2000.0;

fn setup(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>, mut materials: ResMut<Assets<ColorMaterial>>) {
    commands.spawn(Camera2d);

    commands.spawn((
        Mesh2d(meshes.add(Rectangle::new(50.0, 50.0))),
        MeshMaterial2d(materials.add(Color::from(LinearRgba::GREEN))),
        Transform::from_xyz(-200.0, GROUND_Y, 0.0), 
        Player { 
            velocity: 0.0, 
            health: 3, 
            invulnerable_timer: 0.0
        },
    ));
    commands.spawn((
        Text2d::new("HP: 3"),
        TextFont {
            font_size: 40.0,
            ..default()
        },
        TextColor(Color::WHITE),
        Transform::from_xyz(-550.0, 300.0, 1.0), 
        HealthText,
    ));
    commands.spawn((
        Text2d::new("Score: 0"),
        TextFont { font_size: 40.0, ..default() },
        TextColor(Color::Srgba(bevy::color::palettes::css::GOLD)),
        Transform::from_xyz(-550.0, 250.0, 1.0), 
        ScoreText,
    ));
}

fn player_movement(keyboard: Res<ButtonInput<KeyCode>>, time: Res<Time>, mut query: Query<(&mut Transform, &mut Player)>) {
    let delta = time.delta().as_secs_f32();
    for (mut transform, mut player) in &mut query {
        let on_ground = transform.translation.y <= GROUND_Y + 0.1 && transform.translation.y >= GROUND_Y - 0.1;

        if keyboard.just_pressed(KeyCode::Space) && on_ground {
            player.velocity = JUMP_FORCE;
        }

        if !on_ground {
            player.velocity += GRAVITY * delta;
        }

        transform.translation.y += player.velocity * delta;

        if transform.translation.y < GROUND_Y {
            transform.translation.y = GROUND_Y;
            if player.velocity < 0.0 {
                player.velocity = 0.0;
            }
        }
    }
}

fn spawn_obstacles(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    time: Res<Time>,
    mut timer: Local<f32>, 
) {
    *timer += time.delta().as_secs_f32();
    if *timer > 2.0 { 
        *timer = 0.0;
        commands.spawn((
            Mesh2d(meshes.add(Rectangle::new(40.0, 80.0))),
            MeshMaterial2d(materials.add(Color::from(LinearRgba::RED))),
            Transform::from_xyz(600.0, GROUND_Y, 0.0), 
            Obstacle,
            Movable,
        ));
    }
}

fn move_everything(
    mut commands: Commands, 
    mut query: Query<(Entity, &mut Transform), With<Movable>>, 
    time: Res<Time>
) {
    for (entity, mut transform) in &mut query {
        transform.translation.x -= 250.0 * time.delta().as_secs_f32();
        
        if transform.translation.x < -800.0 {
            commands.entity(entity).despawn();
        }
    }
}

fn check_collisions(
    mut commands: Commands,
    time: Res<Time>,
    mut score: ResMut<Score>,
    mut player_query: Query<(&mut Transform, &mut Player, &MeshMaterial2d<ColorMaterial>), With<Player>>,
    obstacle_query: Query<(Entity, &Transform), (With<Obstacle>, Without<Player>)>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let delta = time.delta().as_secs_f32();

    for (mut p_tf, mut player, material_handle) in &mut player_query {
        if player.invulnerable_timer > 0.0 {
            player.invulnerable_timer -= delta;
            
            if let Some(mat) = materials.get_mut(&material_handle.0) {
                mat.color = Color::Srgba(bevy::color::palettes::css::YELLOW);
            }
        } else {
            if let Some(mat) = materials.get_mut(&material_handle.0) {
                mat.color = Color::from(LinearRgba::GREEN);
            }
        }

        if player.invulnerable_timer > 0.0 {
            return;
        }

        for (o_entity, o_tf) in &obstacle_query {
            let p = p_tf.translation;
            let o = o_tf.translation;

            let collision_x = (p.x - o.x).abs() < (50.0 + 40.0) / 2.0;
            let collision_y = (p.y - o.y).abs() < (50.0 + 80.0) / 2.0;

            if collision_x && collision_y {
                player.health -= 1;
                player.invulnerable_timer = 1.0; 
                info!("Straciłeś życie! Pozostało: {}", player.health);

                if player.health <= 0 {
                    info!("💀 KONIEC GRY!");
                    player.health = 3;
                    score.0 = 0;
                    p_tf.translation.y = GROUND_Y;
                    player.velocity = 0.0;
                    
                    for (ent, _) in &obstacle_query {
                        commands.entity(ent).despawn();
                    }
                } else {
                    commands.entity(o_entity).despawn();
                }
            }
        }
    }
}

fn update_health_ui(
    player_query: Query<&Player>,
    mut text_query: Query<&mut Text2d, With<HealthText>>,
) {
    for player in &player_query {
        for mut text in &mut text_query {
            text.0 = format!("HP: {}", player.health);
        }
    }
}

fn spawn_coins(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    time: Res<Time>,
    mut timer: Local<f32>,
) {
    *timer += time.delta().as_secs_f32();
    if *timer > 3.0 { 
        *timer = 0.0;
        let random_y = GROUND_Y + 50.0; 
        
        commands.spawn((
            Mesh2d(meshes.add(Rectangle::new(20.0, 20.0))),
            MeshMaterial2d(materials.add(Color::Srgba(bevy::color::palettes::css::YELLOW))),
            Transform::from_xyz(800.0, random_y, 0.0),
            Coin,
            Movable, 
        ));
    }
}

fn collect_coins(
    mut commands: Commands,
    player_query: Query<&Transform, With<Player>>,
    coin_query: Query<(Entity, &Transform), With<Coin>>,
    mut score: ResMut<Score>,
) {
    for player_tf in &player_query {
        for (coin_entity, coin_tf) in &coin_query {
            let p = player_tf.translation;
            let c = coin_tf.translation;

            let collision = (p.x - c.x).abs() < 35.0 && (p.y - c.y).abs() < 35.0;

            if collision {
                score.0 += 1; 
                commands.entity(coin_entity).despawn(); 
                info!("Zabrano monetę! Wynik: {}", score.0);
            }
        }
    }
}

fn update_ui(
    player_query: Query<&Player>,
    score: Res<Score>,
    mut hp_text: Query<&mut Text2d, (With<HealthText>, Without<ScoreText>)>,
    mut score_text: Query<&mut Text2d, (With<ScoreText>, Without<HealthText>)>,
) {
    for player in &player_query {
        for mut text in &mut hp_text {
            text.0 = format!("HP: {}", player.health);
        }
    }
    for mut text in &mut score_text {
        text.0 = format!("Score: {}", score.0);
    }
}
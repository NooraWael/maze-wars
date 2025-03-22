use bevy::prelude::*;

mod game_front;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                game_front::player::player_look,
                game_front::player::player_movement,
                game_front::world::wall_collision_system,
            ),
        )
        .run();
}

// Set up the scene with world, player, and light
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Create the world (plane, walls, and sky)
    game_front::world::create_world(&mut commands, &mut meshes, &mut materials);

    // Spawn player
    game_front::player::spawn_player(&mut commands, &mut meshes, &mut materials);

    // Directional light (sun)
    commands.spawn((
        DirectionalLight {
            color: Color::srgb(0.98, 0.95, 0.82),
            shadows_enabled: true,
            illuminance: 10000.0,
            ..default()
        },
        Transform::from_xyz(0.0, 10.0, 0.0).looking_at(Vec3::new(-0.15, -0.5, 0.25), Vec3::Y),
    ));

    // Ambient light to make sure everything is visible
    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 0.2,
    });
}

use bevy::{prelude::*, render::camera::Viewport, window::WindowResized};

mod game_front;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .add_systems(PostStartup, tag_player_camera)
        .add_systems(
            Update,
            (
                game_front::player::player_look,
                game_front::player::player_movement,
                game_front::world::wall_collision_system,
                update_camera_viewports,
                update_minimap_camera,
            ),
        )
        .run();
}

// Components for our cameras
#[derive(Component)]
struct MainCamera;

#[derive(Component)]
struct MinimapCamera;

// Resource to store the player entity for the tagging system
#[derive(Resource)]
struct PlayerEntityResource(Entity);

// Set up the scene with world, player, and light
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    game_front::world::create_world(&mut commands, &mut meshes, &mut materials);

    let player_entity =
        game_front::player::spawn_player(&mut commands, &mut meshes, &mut materials);

    commands.insert_resource(PlayerEntityResource(player_entity));

    // Create minimap camera (top-down view - higher angle)
    commands.spawn((
        Camera3d::default(),
        Camera {
            order: 1, // Higher order means it renders on top
            ..default()
        },
        MinimapCamera,
        Transform::from_xyz(0.0, 145.0, 0.0).looking_at(Vec3::ZERO, Vec3::Z),
    ));

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

    // Ambient light
    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 0.2,
    });
}

// System to find and tag the player's camera
fn tag_player_camera(
    mut commands: Commands,
    player_entity: Res<PlayerEntityResource>,
    camera_query: Query<(Entity, &Parent), With<Camera3d>>,
) {
    for (camera_entity, parent) in camera_query.iter() {
        if parent.get() == player_entity.0 {
            commands.entity(camera_entity).insert(MainCamera);
            break;
        }
    }
}

// System to update camera viewports when window is resized
fn update_camera_viewports(
    windows: Query<&Window>,
    mut resize_events: EventReader<WindowResized>,
    mut main_camera: Query<&mut Camera, (With<MainCamera>, Without<MinimapCamera>)>,
    mut minimap_camera: Query<&mut Camera, With<MinimapCamera>>,
) {
    for resize_event in resize_events.read() {
        let window = windows.get(resize_event.window).unwrap();
        let window_width = window.physical_width();
        let window_height = window.physical_height();

        // Main camera takes up the full window
        if let Ok(mut camera) = main_camera.get_single_mut() {
            camera.viewport = Some(Viewport {
                physical_position: UVec2::new(0, 0),
                physical_size: UVec2::new(window_width, window_height),
                ..default()
            });
        }

        // Minimap camera takes up 20% of the window in the top-right corner
        if let Ok(mut camera) = minimap_camera.get_single_mut() {
            let minimap_size = (window_width as f32 * 0.2) as u32;
            camera.viewport = Some(Viewport {
                physical_position: UVec2::new(window_width - minimap_size, 0),
                physical_size: UVec2::new(minimap_size, minimap_size),
                ..default()
            });
        }
    }
}

// System to update the minimap camera position to follow the player - might remove 
fn update_minimap_camera(
    player_query: Query<&Transform, (With<game_front::player::Player>, Without<MinimapCamera>)>,
    mut minimap_camera: Query<&mut Transform, With<MinimapCamera>>,
) {
    if let Ok(player_transform) = player_query.get_single() {
        if let Ok(mut camera_transform) = minimap_camera.get_single_mut() {
            camera_transform.translation.x = player_transform.translation.x;
            camera_transform.translation.z = player_transform.translation.z;
            camera_transform.translation.y = 25.0;
        }
    }
}

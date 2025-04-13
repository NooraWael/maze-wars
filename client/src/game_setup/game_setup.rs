use crate::game_front;
use crate::menu::GameState;
use crate::PlayerEntityResource;
use bevy::prelude::*;
use bevy::render::camera::Viewport;

pub struct GameSetupPlugin;

impl Plugin for GameSetupPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::InGame), setup_game)
            // Add this line to make sure the tagging system runs after setup
            .add_systems(PostStartup, crate::tag_player_camera)
            // Add this line to initialize viewports after setup
            .add_systems(
                OnEnter(GameState::InGame),
                initialize_camera_viewports.after(setup_game),
            )
            .add_systems(OnExit(GameState::InGame), cleanup_game);
    }
}

// This system runs when entering the InGame state
fn setup_game(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    info!("Setting up game world...");

    // Create the game world
    game_front::world::create_world(&mut commands, &mut meshes, &mut materials);

    // Spawn the player
    let player_entity =
        game_front::player::spawn_player(&mut commands, &mut meshes, &mut materials);
    commands.insert_resource(PlayerEntityResource(player_entity));

    // Create minimap camera (top-down view)
    commands.spawn((
        Camera3d::default(),
        Camera {
            order: 1,
            ..default()
        },
        crate::MinimapCamera,
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

// This system runs when exiting the InGame state
fn cleanup_game(
    mut commands: Commands,
    entities: Query<Entity, Without<Camera>>, // Don't remove UI cameras
) {
    info!("Cleaning up game world...");

    // Remove all game entities
    for entity in entities.iter() {
        commands.entity(entity).despawn_recursive();
    }

    // Remove the player entity resource
    commands.remove_resource::<PlayerEntityResource>();
}

// Add this to your game_setup.rs file
fn initialize_camera_viewports(
    windows: Query<&Window>,
    mut main_camera: Query<&mut Camera, (With<crate::MainCamera>, Without<crate::MinimapCamera>)>,
    mut minimap_camera: Query<&mut Camera, With<crate::MinimapCamera>>,
) {
    // Get the primary window
    if let Ok(window) = windows.get_single() {
        let window_width = window.physical_width();
        let window_height = window.physical_height();

        info!(
            "Initializing camera viewports. Window size: {}x{}",
            window_width, window_height
        );

        // Main camera takes up the full window
        if let Ok(mut camera) = main_camera.get_single_mut() {
            camera.viewport = Some(Viewport {
                physical_position: UVec2::new(0, 0),
                physical_size: UVec2::new(window_width, window_height),
                ..default()
            });
            info!("Main camera viewport set to full window");
        } else {
            warn!("Main camera not found when initializing viewports");
        }

        // Minimap camera takes up 20% of the window in the top-right corner
        if let Ok(mut camera) = minimap_camera.get_single_mut() {
            let minimap_size = (window_width as f32 * 0.2) as u32;
            camera.viewport = Some(Viewport {
                physical_position: UVec2::new(window_width - minimap_size, 0),
                physical_size: UVec2::new(minimap_size, minimap_size),
                ..default()
            });
            info!("Minimap camera viewport set to top-right corner");
        } else {
            warn!("Minimap camera not found when initializing viewports");
        }
    }
}

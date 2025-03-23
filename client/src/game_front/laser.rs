use bevy::prelude::*;
use std::time::Duration;

// New component for the laser beam
#[derive(Component)]
pub struct Laser {
    pub lifetime: Timer,
}

impl Default for Laser {
    fn default() -> Self {
        Self {
            lifetime: Timer::new(Duration::from_secs_f32(0.2), TimerMode::Once),
        }
    }
}

// Component to mark the player as having shot recently
#[derive(Component)]
pub struct RecentlyShot {
    pub cooldown: Timer,
}

impl Default for RecentlyShot {
    fn default() -> Self {
        Self {
            cooldown: Timer::new(Duration::from_secs_f32(0.25), TimerMode::Once),
        }
    }
}

// System to handle shooting when 'O' is pressed
pub fn player_shoot(
    mut commands: Commands,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    time: Res<Time>,
    player_query: Query<(Entity, &Transform), With<super::player::Player>>,
    mut recently_shot_query: Query<&mut RecentlyShot>,
) {
    // Check if player can shoot (cooldown expired or doesn't exist)
    let can_shoot = if let Ok(mut recently_shot) = recently_shot_query.get_single_mut() {
        // Update cooldown timer
        recently_shot.cooldown.tick(time.delta());
        recently_shot.cooldown.finished()
    } else {
        true
    };

    // Handle shooting input
    if keyboard_input.just_pressed(KeyCode::KeyO) && can_shoot {
        if let Ok((player_entity, player_transform)) = player_query.get_single() {
            // Calculate laser start position (at the player position)
            let start_pos = player_transform.translation;
            
            // Calculate direction (forward vector from player's rotation)
            let direction = player_transform.forward();
            
            // Calculate end position (20 units forward from player)
            let end_pos = start_pos + (direction * 20.0);
            
            // Create a thin laser mesh (cylinder)
            let laser_mesh = meshes.add(Cylinder::new(0.05, 1.0));  // Thin cylinder
            
            // Bright red glowing material
            let laser_material = materials.add(StandardMaterial {
                base_color: Color::srgb(1.0, 0.0, 0.0),
                emissive: Color::srgb(5.0, 0.0, 0.0).into(),  // Use .into() to convert to LinearRgba // Glow effect (intensity through higher value)
                ..default()
            });
            
            // Calculate length and position
            let laser_length = (end_pos - start_pos).length();
            let laser_position = start_pos + direction * (laser_length / 2.0);
            
            // Calculate rotation to align with the direction
            let laser_rotation = Quat::from_rotation_arc(Vec3::Y, direction.normalize());
            
            // Spawn the laser entity
            commands.spawn((
                Mesh3d(laser_mesh),
                MeshMaterial3d(laser_material),
                Transform {
                    translation: laser_position,
                    rotation: laser_rotation,
                    scale: Vec3::new(1.0, laser_length, 1.0),
                },
                Laser::default(),
            ));
            
            // Add or refresh the recently shot component
            if let Ok(_) = recently_shot_query.get_single() {
                commands.entity(player_entity).remove::<RecentlyShot>();
            }
            
            commands.entity(player_entity).insert(RecentlyShot::default());
        }
    }
}

// System to update and remove lasers based on their lifetime
pub fn update_lasers(
    mut commands: Commands,
    time: Res<Time>,
    mut laser_query: Query<(Entity, &mut Laser)>,
) {
    for (entity, mut laser) in laser_query.iter_mut() {
        laser.lifetime.tick(time.delta());
        
        // Remove laser if lifetime is over
        if laser.lifetime.finished() {
            commands.entity(entity).despawn();
        }
    }
}
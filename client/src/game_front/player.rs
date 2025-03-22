use bevy::prelude::*;
use bevy::input::mouse::MouseMotion;

// Player component
#[derive(Component)]
pub struct Player {
    pub speed: f32,
}

// Camera sensitivity component
#[derive(Component)]
pub struct CameraSensitivity {
    pub horizontal: f32,
    pub vertical: f32,
}

impl Default for CameraSensitivity {
    fn default() -> Self {
        Self {
            horizontal: 0.003,
            vertical: 0.002,
        }
    }
}

// Spawn player function
pub fn spawn_player(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
) {
    // Player entity
    commands.spawn((
        Player { speed: 5.0 },
        CameraSensitivity::default(),
        Mesh3d(meshes.add(Sphere::new(0.5))),
        MeshMaterial3d(materials.add(Color::srgb_u8(255, 100, 100))),
        Transform::from_xyz(0.0, 0.5, 0.0),
    ))
    .with_children(|parent| {
    
        parent.spawn((
            Camera3d::default(),
            Transform::from_xyz(0.0, 4.0, 8.0).looking_at(Vec3::new(0.0, 0.5, -4.0), Vec3::Y),
        ));
    });
}

//i stole this from bevy website
// System to handle player look with mouse
pub fn player_look(
    mut motion_evr: EventReader<MouseMotion>,
    mut query: Query<(&CameraSensitivity, &mut Transform), With<Player>>,
) {
    if let Ok((sensitivity, mut transform)) = query.get_single_mut() {
        let mut delta = Vec2::ZERO;
        
        // Accumulate all mouse motion this frame
        for ev in motion_evr.read() {
            delta += ev.delta;
        }
        
        if delta != Vec2::ZERO {
            // Rotation around Y-axis (yaw)
            let delta_yaw = -delta.x * sensitivity.horizontal;
            // Rotation around X-axis (pitch)
            let delta_pitch = -delta.y * sensitivity.vertical;
            
            // Get current rotation
            let (mut yaw, mut pitch, roll) = transform.rotation.to_euler(EulerRot::YXZ);
            
            // Apply rotation changes
            yaw += delta_yaw;
            
            // Clamp pitch to avoid gimbal lock (looking straight up or down)
            const PITCH_LIMIT: f32 = std::f32::consts::FRAC_PI_2 - 0.01;
            pitch = (pitch + delta_pitch).clamp(-PITCH_LIMIT, PITCH_LIMIT);
            
            // Update rotation
            transform.rotation = Quat::from_euler(EulerRot::YXZ, yaw, pitch, roll);
        }
    }
}

// System to handle player movement with keyboard
pub fn player_movement(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut query: Query<(&Player, &mut Transform)>,
) {
    if let Ok((player, mut transform)) = query.get_single_mut() {
        let mut movement = Vec3::ZERO;
        
        // Forward/backward
        if keyboard_input.pressed(KeyCode::KeyW) || keyboard_input.pressed(KeyCode::ArrowUp) {
            movement.z -= 1.0;
        }
        if keyboard_input.pressed(KeyCode::KeyS) || keyboard_input.pressed(KeyCode::ArrowDown) {
            movement.z += 1.0;
        }
        
        // Left/right
        if keyboard_input.pressed(KeyCode::KeyA) || keyboard_input.pressed(KeyCode::ArrowLeft) {
            movement.x -= 1.0;
        }
        if keyboard_input.pressed(KeyCode::KeyD) || keyboard_input.pressed(KeyCode::ArrowRight) {
            movement.x += 1.0;
        }
        
        // Only apply movement if there's input
        if movement != Vec3::ZERO {
            // Normalize to prevent faster diagonal movement
            movement = movement.normalize();
            
            // Convert movement direction from local space to world space
            // This makes movement relative to where the player is looking
            let forward = transform.forward();
            let right = transform.right();
            
            // Keep movement on the xz plane
            let forward = Vec3::new(forward.x, 0.0, forward.z).normalize_or_zero();
            let right = Vec3::new(right.x, 0.0, right.z).normalize_or_zero();
            
            // Calculate final movement direction
            let direction = forward * -movement.z + right * movement.x;
            
            // Apply movement
            transform.translation += direction * player.speed * time.delta_secs();
        }
    }
}
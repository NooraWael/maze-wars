use bevy::prelude::*;
use bevy::pbr::NotShadowCaster;

// Wall component for collision detection
#[derive(Component)]
pub struct Wall;

// Function to create the game world (plane, walls, and sky)
pub fn create_world(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
) {
    let arena_size = 10.0; 
    
    // Ground plane
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(arena_size * 2.0, arena_size * 2.0))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb_u8(255, 144, 100),
            perceptual_roughness: 1.0,  // does not reflect so we can see it better 
            reflectance: 0.1,           
            ..default()
        })),
        Transform::from_xyz(0.0, -0.01, 0.0),
    ));

    // Wall properties
    let wall_height = 2.0;
    let wall_thickness = 0.5;
    
    // North wall (Z-)
    commands.spawn((
        Wall,
        Mesh3d(meshes.add(Cuboid::new(arena_size * 2.0, wall_height, wall_thickness))),
        MeshMaterial3d(materials.add(Color::srgb_u8(180, 180, 180))),
        Transform::from_xyz(0.0, wall_height / 2.0, -arena_size),
    ));

    // South wall (Z+)
    commands.spawn((
        Wall,
        Mesh3d(meshes.add(Cuboid::new(arena_size * 2.0, wall_height, wall_thickness))),
        MeshMaterial3d(materials.add(Color::srgb_u8(180, 180, 180))),
        Transform::from_xyz(0.0, wall_height / 2.0, arena_size),
    ));

    // East wall (X+)
    commands.spawn((
        Wall,
        Mesh3d(meshes.add(Cuboid::new(wall_thickness, wall_height, arena_size * 2.0))),
        MeshMaterial3d(materials.add(Color::srgb_u8(180, 180, 180))),
        Transform::from_xyz(arena_size, wall_height / 2.0, 0.0),
    ));

    // West wall (X-)
    commands.spawn((
        Wall,
        Mesh3d(meshes.add(Cuboid::new(wall_thickness, wall_height, arena_size * 2.0))),
        MeshMaterial3d(materials.add(Color::srgb_u8(180, 180, 180))),
        Transform::from_xyz(-arena_size, wall_height / 2.0, 0.0),
    ));
    
    // Sky (large cube with inverted normals)
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(1.0, 1.0, 1.0))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.5, 0.7, 1.0), 
            unlit: true,                           
            cull_mode: None,                       
            ..default()
        })),
        Transform::from_scale(Vec3::splat(50.0)), 
        NotShadowCaster,                     
    ));
}

// System to handle collision detection with walls
pub fn wall_collision_system(
    mut player_query: Query<&mut Transform, (With<crate::game_front::player::Player>, Without<Wall>)>,
) {
    let arena_size = 9.5; // Slightly smaller than actual arena to account for player size
    
    for mut player_transform in player_query.iter_mut() {
        // Simple boundary collision - keep player within arena bounds
        player_transform.translation.x = player_transform.translation.x.clamp(-arena_size, arena_size);
        player_transform.translation.z = player_transform.translation.z.clamp(-arena_size, arena_size);
    }
}


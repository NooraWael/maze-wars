use bevy::prelude::*;
use bevy::math::primitives::Capsule3d;
use crate::game_front::player::Player;
use crate::network::network::{
    OutgoingMessages, ClientMessage, ServerMessage, 
    Position, Rotation, send_message, NetworkEvent
};

// Component for other networked players
#[derive(Component)]
pub struct NetworkedPlayer {
    pub player_id: String,
}

pub struct PlayerNetworkPlugin;

impl Plugin for PlayerNetworkPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                send_player_movement,
                send_player_actions,
                handle_network_player_updates,
            ).run_if(in_state(crate::menu::GameState::InGame)),
        );
    }
}

// System to send player movement data to server
fn send_player_movement(
    player_query: Query<(&Transform, &Player), Changed<Transform>>,
    outgoing_msgs: Res<OutgoingMessages>,
) {
    // Only send updates when the player's transform has changed
    if let Ok((transform, _)) = player_query.get_single() {
        let position = Position::from(transform.translation);
        let rotation = Rotation::from(transform.rotation);
        
        // Send the movement update
        let _ = send_message(ClientMessage::Move {
            position,
            rotation,
            yield_control: 0.0, // Not sure what this is used for in your game
        }, &outgoing_msgs);
    }
}

// System to send player actions (shooting, etc.)
fn send_player_actions(
    player_query: Query<&Transform, With<Player>>,
    keyboard: Res<ButtonInput<KeyCode>>,
    outgoing_msgs: Res<OutgoingMessages>,
) {
    if let Ok(transform) = player_query.get_single() {
        if keyboard.just_pressed(KeyCode::KeyO) {
            let position = Position::from(transform.translation);
            let rotation = Rotation::from(transform.rotation);
            
            let _ = send_message(ClientMessage::Shoot {
                position,
                direction: rotation,
                weapon_type: "standard".to_string(),
            }, &outgoing_msgs);
        }
        
        // Add more actions as needed - TODO
    }
}

// System to handle network events related to other players
fn handle_network_player_updates(
    mut commands: Commands,
    mut network_events: EventReader<NetworkEvent>,
    mut networked_players: Query<(Entity, &mut Transform, &NetworkedPlayer)>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for event in network_events.read() {
        if let NetworkEvent::ReceivedMessage(server_msg) = event {
            match server_msg {
                ServerMessage::PlayerMove { player_id, position, rotation, .. } => {
                    // Update or create other player
                    update_networked_player(
                        &mut commands,
                        &mut networked_players,
                        player_id,
                        *position,
                        *rotation,
                        &mut meshes,
                        &mut materials,
                    );
                },
                ServerMessage::PlayerSpawn { player_id, position } => {
                    // Spawn a new networked player
                    spawn_networked_player(
                        &mut commands,
                        player_id,
                        *position,
                        Rotation { pitch: 0.0, yaw: 0.0, roll: 0.0 },
                        &mut meshes,
                        &mut materials,
                    );
                },
                ServerMessage::PlayerDeath { player_id, .. } => {
                    // Remove the player entity
                    for (entity, _, networked_player) in networked_players.iter() {
                        if networked_player.player_id == *player_id {
                            commands.entity(entity).despawn_recursive();
                            break;
                        }
                    }
                },
                // Handle other messages as needed
                _ => {}
            }
        }
    }
}

// Helper function to update or create a networked player
fn update_networked_player(
    commands: &mut Commands,
    networked_players: &mut Query<(Entity, &mut Transform, &NetworkedPlayer)>,
    player_id: &String,
    position: Position,
    rotation: Rotation,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
) {
    // Try to find an existing player with this ID
    let mut found = false;
    for (_, mut transform, networked_player) in networked_players.iter_mut() {
        if networked_player.player_id == *player_id {
            // Update the player's position and rotation
            transform.translation = Vec3::new(position.x, position.y, position.z);
            transform.rotation = rotation.into();
            found = true;
            break;
        }
    }

    // If we didn't find the player, create a new one
    if !found {
        spawn_networked_player(
            commands,
            player_id,
            position,
            rotation,
            meshes,
            materials,
        );
    }
}

// Helper function to spawn a new networked player
fn spawn_networked_player(
    commands: &mut Commands,
    player_id: &String,
    position: Position,
    rotation: Rotation,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
) {
    // Create a mesh for other players with updated component system
    commands.spawn((
        // Using the new component system instead of deprecated PbrBundle
        Mesh3d(meshes.add(Mesh::from(Capsule3d::default()))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.8, 0.2, 0.2),
            ..default()
        })),
        Transform::from_translation(Vec3::new(position.x, position.y, position.z))
            .with_rotation(rotation.into()),
        NetworkedPlayer {
            player_id: player_id.clone(),
        },
    ));
}
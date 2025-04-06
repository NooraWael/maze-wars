use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::UdpSocket;
use tokio::runtime::Runtime;
use tokio::sync::{mpsc, Mutex};

// Plugin for the network functionality
pub struct NetworkPlugin;

impl Plugin for NetworkPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<NetworkState>()
            .init_resource::<OutgoingMessages>()
            .init_resource::<IncomingMessages>()
            .add_event::<NetworkEvent>()
            .add_systems(
                Update,
                (
                    process_outgoing_messages,
                    process_incoming_messages,
                    handle_network_events,
                ),
            );
    }
}

// Client messages to send to the server
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type", content = "data")]
pub enum ClientMessage {
    JoinGame {
        username: String,
    },
    Move {
        position: Position,
        rotation: Rotation,
        yield_control: f32,
    },
    Shoot {
        position: Position,
        direction: Rotation,
        weapon_type: String,
    },
}

// Messages received from the server
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type", content = "data")]
pub enum ServerMessage {
    Error {
        message: String,
    },
    GameStart,
    PlayersInLobby {
        player_count: u32,
        players: Vec<String>,
    },
    PlayerMove {
        player_id: String,
        position: Position,
        rotation: Rotation,
        yield_control: f32,
    },
    PlayerShoot {
        player_id: String,
        position: Position,
        direction: Rotation,
        weapon_type: String,
    },
    PlayerDeath {
        player_id: String,
        killer_id: Option<String>,
    },
    PlayerSpawn {
        player_id: String,
        position: Position,
    },
    HealthUpdate {
        player_id: String,
        health: u32,
    },
}

// Position in 3D space
#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub struct Position {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl From<Vec3> for Position {
    fn from(vec: Vec3) -> Self {
        Self {
            x: vec.x,
            y: vec.y,
            z: vec.z,
        }
    }
}

impl Into<Vec3> for Position {
    fn into(self) -> Vec3 {
        Vec3::new(self.x, self.y, self.z)
    }
}

// Rotation in 3D space
#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub struct Rotation {
    pub pitch: f32,
    pub yaw: f32,
    pub roll: f32,
}

impl From<Quat> for Rotation {
    fn from(quat: Quat) -> Self {
        let (yaw, pitch, roll) = quat.to_euler(EulerRot::YXZ);
        Self { pitch, yaw, roll }
    }
}

impl Into<Quat> for Rotation {
    fn into(self) -> Quat {
        Quat::from_euler(EulerRot::YXZ, self.yaw, self.pitch, self.roll)
    }
}

// Network events for game systems
#[derive(Event, Debug)]
pub enum NetworkEvent {
    Connected,
    Disconnected,
    Error(String),
    ReceivedMessage(ServerMessage),
}

// State of the network connection
#[derive(Resource, Default)]
pub struct NetworkState {
    pub connected: bool,
    pub server_addr: Option<SocketAddr>,
    pub username: Option<String>,
}

// Outgoing message queue
#[derive(Resource)]
pub struct OutgoingMessages {
    pub sender: Option<mpsc::Sender<ClientMessage>>,
}

impl Default for OutgoingMessages {
    fn default() -> Self {
        Self { sender: None }
    }
}

// Incoming message queue
#[derive(Resource)]
pub struct IncomingMessages {
    pub receiver: Option<mpsc::Receiver<ServerMessage>>,
}

impl Default for IncomingMessages {
    fn default() -> Self {
        Self { receiver: None }
    }
}

// Connect to the server
pub fn connect_to_server(
    server_addr: &str,
    username: String,
    outgoing_msgs: &mut ResMut<OutgoingMessages>,
    incoming_msgs: &mut ResMut<IncomingMessages>,
    network_state: &mut ResMut<NetworkState>,
) -> Result<(), String> {
    // Parse the server address
    let addr = server_addr
        .parse::<SocketAddr>()
        .map_err(|e| format!("Invalid server address: {}", e))?;

    // Create message channels
    let (outgoing_tx, mut outgoing_rx) = mpsc::channel::<ClientMessage>(100);
    let (incoming_tx, incoming_rx) = mpsc::channel::<ServerMessage>(100);

    // Set up the network state
    network_state.server_addr = Some(addr);
    network_state.username = Some(username.clone());

    // Store channel endpoints
    outgoing_msgs.sender = Some(outgoing_tx);
    incoming_msgs.receiver = Some(incoming_rx);

    // Create a Tokio runtime
    let rt = Runtime::new().map_err(|e| format!("Failed to create runtime: {}", e))?;

    // Spawn the network thread
    std::thread::spawn(move || {
        rt.block_on(async move {
            // Create UDP socket
            let socket = match UdpSocket::bind("0.0.0.0:0").await {
                Ok(s) => Arc::new(s),
                Err(e) => {
                    let _ = incoming_tx
                        .send(ServerMessage::Error {
                            message: format!("Failed to bind socket: {}", e),
                        })
                        .await;
                    return;
                }
            };

            // Connect to the server
            if let Err(e) = socket.connect(addr).await {
                let _ = incoming_tx
                    .send(ServerMessage::Error {
                        message: format!("Failed to connect: {}", e),
                    })
                    .await;
                return;
            }

            // Send initial join message
            let join_msg = ClientMessage::JoinGame { username };
            let json = match serde_json::to_string(&join_msg) {
                Ok(j) => j,
                Err(e) => {
                    let _ = incoming_tx
                        .send(ServerMessage::Error {
                            message: format!("Failed to serialize join message: {}", e),
                        })
                        .await;
                    return;
                }
            };

            if let Err(e) = socket.send(json.as_bytes()).await {
                let _ = incoming_tx
                    .send(ServerMessage::Error {
                        message: format!("Failed to send join message: {}", e),
                    })
                    .await;
                return;
            }

            // Clone socket for the receiver task
            let recv_socket = socket.clone();

            // Spawn a task to handle incoming messages
            let incoming_task = tokio::spawn(async move {
                let mut buf = vec![0u8; 2048];
                loop {
                    match recv_socket.recv(&mut buf).await {
                        Ok(len) => {
                            if len > 0 {
                                let data = &buf[..len];
                                if let Ok(msg) = serde_json::from_slice::<ServerMessage>(data) {
                                    if incoming_tx.send(msg).await.is_err() {
                                        break;
                                    }
                                }
                            }
                        }
                        Err(_) => break,
                    }
                }
            });

            // Handle outgoing messages
            let outgoing_task = tokio::spawn(async move {
                while let Some(msg) = outgoing_rx.recv().await {
                    if let Ok(json) = serde_json::to_string(&msg) {
                        let _ = socket.send(json.as_bytes()).await;
                    }
                }
            });

            // Wait for either task to complete
            tokio::select! {
                _ = incoming_task => {},
                _ = outgoing_task => {},
            }
        });
    });

    Ok(())
}

// System to process outgoing messages
fn process_outgoing_messages(
    player_query: Query<(&Transform, &crate::game_front::player::Player)>,
    keyboard: Res<ButtonInput<KeyCode>>,
    outgoing_msgs: Res<OutgoingMessages>,
    network_state: Res<NetworkState>,
) {
    if !network_state.connected || outgoing_msgs.sender.is_none() {
        return;
    }

    if let Some(sender) = &outgoing_msgs.sender {
        // Only send messages if we have a player
        if let Ok((transform, _)) = player_query.get_single() {
            // Send position updates
            let position = Position::from(transform.translation);
            let rotation = Rotation::from(transform.rotation);

            // For now, just send position updates - this can be optimized to only send when needed
            let _ = sender.try_send(ClientMessage::Move {
                position,
                rotation,
                yield_control: 0.0,
            });

            // Handle shooting
            if keyboard.just_pressed(KeyCode::KeyO) {
                let _ = sender.try_send(ClientMessage::Shoot {
                    position,
                    direction: rotation,
                    weapon_type: "standard".to_string(),
                });
            }
        }
    }
}

// System to process incoming messages
fn process_incoming_messages(
    mut incoming_msgs: ResMut<IncomingMessages>,
    mut network_events: EventWriter<NetworkEvent>,
) {
    if let Some(receiver) = &mut incoming_msgs.receiver {
        // Try to get all available messages
        while let Ok(Some(msg)) = receiver.try_recv().map(Option::Some) {
            network_events.send(NetworkEvent::ReceivedMessage(msg));
        }
    }
}

// System to handle network events
fn handle_network_events(
    mut events: EventReader<NetworkEvent>,
    mut network_state: ResMut<NetworkState>,
) {
    for event in events.read() {
        match event {
            NetworkEvent::Connected => {
                network_state.connected = true;
                info!("Connected to server!");
            }
            NetworkEvent::Disconnected => {
                network_state.connected = false;
                info!("Disconnected from server");
            }
            NetworkEvent::Error(message) => {
                error!("Network error: {}", message);
            }
            NetworkEvent::ReceivedMessage(msg) => {
                match msg {
                    ServerMessage::Error { message } => {
                        error!("Server error: {}", message);
                    }
                    ServerMessage::GameStart => {
                        info!("Game started!");
                    }
                    ServerMessage::PlayersInLobby {
                        player_count,
                        players,
                    } => {
                        info!("Players in lobby: {} ({:?})", player_count, players);
                    }
                    ServerMessage::PlayerMove {
                        player_id,
                        position,
                        rotation,
                        ..
                    } => {
                        // Update other player positions
                        // This would need to be handled by a separate system managing other players
                        debug!("Player {} moved to {:?}", player_id, position);
                    }
                    ServerMessage::PlayerShoot { player_id, .. } => {
                        // Handle visual effects for other players shooting
                        debug!("Player {} fired weapon", player_id);
                    }
                    ServerMessage::PlayerDeath {
                        player_id,
                        killer_id,
                    } => {
                        // Handle player death events
                        if let Some(killer) = killer_id {
                            info!("Player {} was killed by {}", player_id, killer);
                        } else {
                            info!("Player {} died", player_id);
                        }
                    }
                    ServerMessage::PlayerSpawn {
                        player_id,
                        position,
                    } => {
                        // Handle player spawn events
                        info!("Player {} spawned at {:?}", player_id, position);
                    }
                    ServerMessage::HealthUpdate { player_id, health } => {
                        // Update player health
                        debug!("Player {} health: {}", player_id, health);
                    }
                }
            }
        }
    }
}

// Helper function to send a message to the server
pub fn send_message(
    message: ClientMessage,
    outgoing_msgs: &OutgoingMessages,
) -> Result<(), String> {
    if let Some(sender) = &outgoing_msgs.sender {
        sender
            .try_send(message)
            .map_err(|e| format!("Failed to send message: {}", e))?;
        Ok(())
    } else {
        Err("Not connected to server".to_string())
    }
}

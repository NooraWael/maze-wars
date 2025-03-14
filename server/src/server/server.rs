use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::UdpSocket;
use tokio::sync::Mutex;

/// Position in 3D space
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Position {
    x: f32,
    y: f32,
    z: f32,
}

/// Rotation in 3D space
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Rotation {
    pitch: f32,
    yaw: f32,
    roll: f32,
}

/// Player information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Player {
    username: String,
    position: Position,
    rotation: Rotation,
    health: u8,
    is_alive: bool,
}

/// Game state to track players and game state
#[derive(Debug)]
struct GameState {
    players: HashMap<SocketAddr, Player>,
    is_game_started: bool,
}

/// Client to Server Messages
#[derive(Debug, Serialize, Deserialize)]
enum ClientMessage {
    /// Join game request with player's username
    JoinGame { username: String },

    /// Player movement update with position, rotation, and yield flag
    Move {
        position: Position,
        rotation: Rotation,
        yield_control: bool,
    },

    /// Player shooting action
    Shoot {
        direction: Rotation,
        weapon_type: String,
    },
}

/// Server to Client Messages
#[derive(Debug, Serialize, Deserialize)]
enum ServerMessage {
    /// Notifies clients that the game has started
    GameStart,

    /// Updates clients about players in the lobby
    PlayersInLobby {
        player_count: u8,
        players: Vec<String>,
    },

    /// Broadcasts player movements to all clients
    PlayerMove {
        player_id: String,
        position: Position,
        rotation: Rotation,
        yield_control: bool,
    },

    /// Broadcasts shooting actions to all clients
    PlayerShoot {
        player_id: String,
        position: Position,
        direction: Rotation,
        weapon_type: String,
    },

    /// Notifies when a player dies
    PlayerDeath {
        player_id: String,
        killer_id: Option<String>,
    },

    /// Notifies when a player spawns
    PlayerSpawn {
        player_id: String,
        position: Position,
    },

    /// Updates clients about player health
    HealthUpdate { player_id: String, health: u8 },
}

#[derive(Debug)]
pub struct Server {
    host: String,
    port: u16,
    min_players: u8,
    max_players: u8,
    game_state: Arc<Mutex<GameState>>,
}

impl Server {
    pub fn new<S: Into<String>>(host: S, port: u16) -> Server {
        Server {
            host: host.into(),
            port,
            min_players: 1,
            max_players: 10,
            game_state: Arc::new(Mutex::new(GameState {
                players: HashMap::new(),
                is_game_started: false,
            })),
        }
    }

    pub fn min_players(&mut self, min: u8) -> &mut Self {
        self.min_players = min;
        self
    }

    pub fn max_players(&mut self, max: u8) -> &mut Self {
        self.max_players = max;
        self
    }

    pub fn start(&mut self) -> &mut Self {
        println!("Starting server at {}:{}", self.host, self.port);

        // Create a runtime for our Tokio async code
        let rt = tokio::runtime::Runtime::new().unwrap();
        let game_state = self.game_state.clone();
        let host = self.host.clone();
        let port = self.port;
        let min_players = self.min_players;
        let max_players = self.max_players;

        // Spawn the UDP server in a background task
        rt.spawn(async move {
            if let Err(e) = run_udp_server(host, port, min_players, max_players, game_state).await {
                eprintln!("Server error: {}", e);
            }
        });

        self
    }
}

/// Main UDP server function
async fn run_udp_server(
    host: String,
    port: u16,
    min_players: u8,
    max_players: u8,
    game_state: Arc<Mutex<GameState>>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Bind the UDP socket
    let socket_addr = format!("{}:{}", host, port);
    let socket = UdpSocket::bind(&socket_addr).await?;
    let socket = Arc::new(socket);

    println!("UDP server listening on {}", socket_addr);

    // Buffer for incoming messages
    let mut buf = [0u8; 1024];

    // Main server loop
    loop {
        let (size, client_addr) = socket.recv_from(&mut buf).await?;

        // Process the received message
        if let Ok(message) = bincode::deserialize::<ClientMessage>(&buf[..size]) {
            println!("Received message from {}: {:?}", client_addr, message);

            match message {
                ClientMessage::JoinGame { username } => {
                    handle_join_game(
                        socket.clone(),
                        client_addr,
                        username,
                        game_state.clone(),
                        max_players,
                    )
                    .await?;
                }
                ClientMessage::Move {
                    position,
                    rotation,
                    yield_control,
                } => {
                    handle_move(
                        socket.clone(),
                        client_addr,
                        position,
                        rotation,
                        yield_control,
                        game_state.clone(),
                    )
                    .await?;
                }
                ClientMessage::Shoot {
                    direction,
                    weapon_type,
                } => {
                    handle_shoot(
                        socket.clone(),
                        client_addr,
                        direction,
                        weapon_type,
                        game_state.clone(),
                    )
                    .await?;
                }
            }

            // Check if the game should start
            let should_start_game = {
                let state = game_state.lock().await;
                !state.is_game_started && state.players.len() >= min_players as usize
            };

            if should_start_game {
                start_game(socket.clone(), game_state.clone()).await?;
            }
        }
    }
}

/// Handle join game request
async fn handle_join_game(
    socket: Arc<UdpSocket>,
    client_addr: SocketAddr,
    username: String,
    game_state: Arc<Mutex<GameState>>,
    max_players: u8,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut state = game_state.lock().await;

    // Check if the game is already full
    if state.players.len() >= max_players as usize {
        // Send rejection message (would need to be implemented)
        return Ok(());
    }

    // Create a new player
    let player = Player {
        username: username.clone(),
        position: Position {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        },
        rotation: Rotation {
            pitch: 0.0,
            yaw: 0.0,
            roll: 0.0,
        },
        health: 100,
        is_alive: true,
    };

    // Add player to the game
    state.players.insert(client_addr, player);

    // Get list of players for lobby update
    let player_count = state.players.len() as u8;
    let players: Vec<String> = state.players.values().map(|p| p.username.clone()).collect();

    // Drop the lock before sending messages
    drop(state);

    // Send players in lobby update to all players
    broadcast_to_all(
        &socket,
        &ServerMessage::PlayersInLobby {
            player_count,
            players,
        },
        game_state,
    )
    .await?;

    Ok(())
}

/// Handle player movement
async fn handle_move(
    socket: Arc<UdpSocket>,
    client_addr: SocketAddr,
    position: Position,
    rotation: Rotation,
    yield_control: bool,
    game_state: Arc<Mutex<GameState>>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut state = game_state.lock().await;

    // Update player position if they exist
    if let Some(player) = state.players.get_mut(&client_addr) {
        player.position = position;
        player.rotation = rotation;

        // Get player ID
        let player_id = player.username.clone();

        // Drop the lock before sending messages
        drop(state);

        // Broadcast move to all players
        broadcast_to_all(
            &socket,
            &ServerMessage::PlayerMove {
                player_id,
                position,
                rotation,
                yield_control,
            },
            game_state,
        )
        .await?;
    }

    Ok(())
}

/// Handle shooting action
async fn handle_shoot(
    socket: Arc<UdpSocket>,
    client_addr: SocketAddr,
    direction: Rotation,
    weapon_type: String,
    game_state: Arc<Mutex<GameState>>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let state = game_state.lock().await;

    // Process shooting only if player exists
    if let Some(player) = state.players.get(&client_addr) {
        let player_id = player.username.clone();
        let position = player.position;

        // Drop the lock before sending messages
        drop(state);

        // Broadcast shoot to all players
        broadcast_to_all(
            &socket,
            &ServerMessage::PlayerShoot {
                player_id,
                position,
                direction,
                weapon_type,
            },
            game_state,
        )
        .await?;

        // In a real game, we'd also calculate hit detection here
        // and send PlayerDeath and HealthUpdate messages as needed
    }

    Ok(())
}

/// Start the game when minimum player count is reached
async fn start_game(
    socket: Arc<UdpSocket>,
    game_state: Arc<Mutex<GameState>>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut state = game_state.lock().await;

    if state.is_game_started {
        return Ok(());
    }

    state.is_game_started = true;
    let player_ids: Vec<(String, Position)> = state
        .players
        .values()
        .map(|p| (p.username.clone(), p.position))
        .collect();

    // Drop the lock before sending messages
    drop(state);

    // Notify all clients that the game has started
    broadcast_to_all(&socket, &ServerMessage::GameStart, game_state.clone()).await?;

    // Spawn all players at their positions
    for (player_id, position) in player_ids {
        broadcast_to_all(
            &socket,
            &ServerMessage::PlayerSpawn {
                player_id,
                position,
            },
            game_state.clone(),
        )
        .await?;
    }

    Ok(())
}

/// Broadcast a message to all connected players
async fn broadcast_to_all(
    socket: &Arc<UdpSocket>,
    message: &ServerMessage,
    game_state: Arc<Mutex<GameState>>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let encoded = bincode::serialize(message)?;
    let state = game_state.lock().await;

    for client_addr in state.players.keys() {
        socket.send_to(&encoded, client_addr).await?;
    }

    Ok(())
}

/// Update player health and handle death if needed
async fn update_player_health(
    socket: Arc<UdpSocket>,
    player_id: String,
    new_health: u8,
    game_state: Arc<Mutex<GameState>>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut state = game_state.lock().await;
    let mut player_died = false;
    let mut player_addr = None;

    // Find and update the player's health
    for (addr, player) in state.players.iter_mut() {
        if player.username == player_id {
            player.health = new_health;

            // Check if player died from this health change
            if new_health == 0 && player.is_alive {
                player.is_alive = false;
                player_died = true;
                player_addr = Some(*addr);
            }
            break;
        }
    }

    // Drop the lock before sending messages
    drop(state);

    // Broadcast health update
    broadcast_to_all(
        &socket,
        &ServerMessage::HealthUpdate {
            player_id: player_id.clone(),
            health: new_health,
        },
        game_state.clone(),
    )
    .await?;

    // Handle player death if needed
    if player_died {
        broadcast_to_all(
            &socket,
            &ServerMessage::PlayerDeath {
                player_id,
                killer_id: None, // In a real game, you'd track the killer
            },
            game_state,
        )
        .await?;
    }

    Ok(())
}

use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::UdpSocket;
use tokio::sync::Mutex;

use crate::player::Player;
use crate::position::Position;
use crate::rotation::Rotation;
use crate::server::client_messages::ClientMessage;
use crate::weapon::Weapon;

use super::ServerMessage;

#[derive(Debug)]
pub struct GameState {
    players: HashMap<SocketAddr, Player>,
}

#[derive(Debug)]
/// Main game server handling network communication and game state
///
/// # Examples
/// ```rust
/// let mut server = Server::new("127.0.0.1", 8080)
///     .min_players(2)
///     .max_players(16);
/// ```
pub struct Server {
    host: String,
    port: u16,
    min_players: u8,
    max_players: u8,
    game_state: Arc<Mutex<GameState>>,
}

impl Server {
    /// Sends a message to a specific player
    ///
    /// # Arguments
    /// * `socket` - Reference to UDP socket
    /// * `message` - ServerMessage to be serialized and sent
    /// * `addr` - Player's network address
    ///
    /// # Returns
    /// Result indicating success or failure
    async fn send_message(
        &self,
        socket: &Arc<UdpSocket>,
        message: ServerMessage,
        addr: &SocketAddr,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let json = serde_json::to_string(&message)?;
        log::trace!("Sending message to {}", addr);
        socket.send_to(json.as_bytes(), addr).await?;
        Ok(())
    }

    /// Broadcasts a message to all connected players
    ///
    /// # Arguments
    /// * `socket` - Reference to UDP socket
    /// * `message` - ServerMessage to be serialized and broadcasted
    /// * `players` - Map of connected players
    ///
    /// # Returns
    /// Result indicating success or failure
    async fn broadcast_message(
        &self,
        socket: &Arc<UdpSocket>,
        message: ServerMessage,
        players: &HashMap<SocketAddr, Player>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        log::trace!("Broadcasting message to {} players", players.len());
        for client_addr in players.keys() {
            self.send_message(socket, message.clone(), client_addr)
                .await?;
        }
        Ok(())
    }

    /// Handles new player joining the game
    ///
    /// # Arguments
    /// * `game_state` - Shared game state
    /// * `socket` - UDP socket reference
    /// * `addr` - Client's network address
    /// * `username` - Player's chosen name
    ///
    /// # Returns
    /// Result indicating success or failure
    async fn handle_join_game(
        &self,
        game_state: Arc<Mutex<GameState>>,
        socket: Arc<UdpSocket>,
        addr: SocketAddr,
        username: String,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut state = game_state.lock().await;
        let player = Player::new(
            username.clone(),
            Default::default(),
            Default::default(),
            100,
            Weapon::pistol(),
        );
        state.players.insert(addr, player);
        log::info!("New player connection: {} from {}", username, addr);

        let response = ServerMessage::PlayersInLobby {
            player_count: state.players.len() as u32,
            players: state.players.values().map(|p| p.username.clone()).collect(),
        };
        let json = serde_json::to_string(&response)?;
        socket.send_to(json.as_bytes(), &addr).await?;
        Ok(())
    }

    /// Processes player movement updates
    ///
    /// # Arguments
    /// * `game_state` - Shared game state
    /// * `socket` - UDP socket reference
    /// * `addr` - Client's network address
    /// * `position` - New 3D position
    /// * `rotation` - New orientation
    /// * `yield_control` - Movement control value
    ///
    /// # Returns
    /// Result indicating success or failure
    async fn handle_move(
        &self,
        game_state: Arc<Mutex<GameState>>,
        socket: Arc<UdpSocket>,
        addr: SocketAddr,
        position: Position,
        rotation: Rotation,
        yield_control: f32,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut state = game_state.lock().await;
        if let Some(player) = state.players.get_mut(&addr) {
            player.position = position;
            player.rotation = rotation;
            log::debug!(
                "Player {} moved to {:?} facing {:?}",
                player.username,
                position,
                rotation
            );

            let response = ServerMessage::PlayerMove {
                player_id: player.username.clone(),
                position: player.position,
                rotation: player.rotation,
                yield_control,
            };
            self.broadcast_message(&socket, response, &state.players)
                .await?;
        }
        Ok(())
    }

    /// Processes player shooting actions
    ///
    /// # Arguments
    /// * `game_state` - Shared game state
    /// * `socket` - UDP socket reference
    /// * `addr` - Client's network address
    /// * `position` - Shot origin position
    /// * `direction` - Shooting direction
    /// * `weapon_type` - Weapon identifier string
    ///
    /// # Returns
    /// Result indicating success or failure
    async fn handle_shoot(
        &self,
        game_state: Arc<Mutex<GameState>>,
        socket: Arc<UdpSocket>,
        addr: SocketAddr,
        position: Position,
        direction: Rotation,
        weapon_type: String,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let state = game_state.lock().await;
        if let Some(player) = state.players.get(&addr) {
            log::debug!(
                "Player {} fired {} weapon from {:?}",
                player.username,
                weapon_type,
                position
            );
            let response = ServerMessage::PlayerShoot {
                player_id: player.username.clone(),
                position,
                direction,
                weapon_type,
            };
            self.broadcast_message(&socket, response, &state.players)
                .await?;
        }
        Ok(())
    }
    pub fn new<S: Into<String>>(host: S, port: u16) -> Server {
        Server {
            host: host.into(),
            port,
            min_players: 1,
            max_players: 10,
            game_state: Arc::new(Mutex::new(GameState {
                players: HashMap::new(),
            })),
        }
    }

    /// Sets minimum required players to start a match
    ///
    /// # Arguments
    /// * `min` - Minimum players (1-255)
    ///
    /// # Returns
    /// Mutable Self for method chaining
    pub fn min_players(&mut self, min: u8) -> &mut Self {
        self.min_players = min;
        self
    }

    /// Sets maximum allowed players in a match
    ///
    /// # Arguments
    /// * `max` - Maximum players (1-255)
    ///
    /// # Returns
    /// Mutable Self for method chaining
    pub fn max_players(&mut self, max: u8) -> &mut Self {
        self.max_players = max;
        self
    }

    /// Starts the game server and begins listening for UDP packets
    ///
    /// # Returns
    /// Result indicating success or failure
    ///
    /// # Examples
    /// ```rust
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let mut server = Server::new("0.0.0.0", 8080);
    ///     server.start().await?;
    ///     Ok(())
    /// }
    /// ```
    pub async fn start(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let addr = format!("{}:{}", self.host, self.port);
        let socket = UdpSocket::bind(&addr).await?;
        log::info!("Server started on {}", addr);

        let socket = Arc::new(socket);
        let game_state = self.game_state.clone();

        let mut buf = vec![0u8; 1024];
        loop {
            log::trace!("Waiting for incoming packets...");
            let (len, addr) = socket.recv_from(&mut buf).await?;
            let message = String::from_utf8_lossy(&buf[..len]);

            log::trace!("Received message from {}: {}", addr, message);

            let client_message = serde_json::from_str::<ClientMessage>(&message);

            if let Err(e) = client_message {
                log::warn!("Failed to parse client message: {}", e);

                let error_message = ServerMessage::Error {
                    message: format!("Bad Payload: {}", e),
                };

                self.send_message(&socket, error_message, &addr).await?;
                continue;
            }

            let client_message = client_message.unwrap();

            match client_message {
                ClientMessage::JoinGame { username } => {
                    self.handle_join_game(game_state.clone(), socket.clone(), addr, username)
                        .await?;
                }
                ClientMessage::Move {
                    position,
                    rotation,
                    yield_control,
                } => {
                    self.handle_move(
                        game_state.clone(),
                        socket.clone(),
                        addr,
                        position,
                        rotation,
                        yield_control,
                    )
                    .await?;
                }
                ClientMessage::Shoot {
                    position,
                    direction,
                    weapon_type,
                } => {
                    self.handle_shoot(
                        game_state.clone(),
                        socket.clone(),
                        addr,
                        position,
                        direction,
                        weapon_type,
                    )
                    .await?;
                }
            }
        }
    }
}

use super::game_state::GameState;
use super::ServerMessage;

use crate::server::client_messages::ClientMessage;

use std::collections::HashMap;
use std::sync::Arc;
use tokio::net::UdpSocket;
use tokio::sync::Mutex;

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
    pub min_players: u8,
    pub max_players: u8,
    game_state: Arc<Mutex<GameState>>,
}

impl Server {
    pub fn new<S: Into<String>>(host: S, port: u16) -> Server {
        Server {
            host: host.into(),
            port,
            min_players: 1,
            max_players: 10,
            game_state: Arc::new(Mutex::new(GameState::new())),
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

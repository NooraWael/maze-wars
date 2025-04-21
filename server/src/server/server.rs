use super::game::Game;

use shared::server::{ClientMessage, ServerMessage};
use std::sync::Arc;
use std::time::Instant;
use tokio::net::UdpSocket;
use tokio::sync::Mutex;
use tokio::time::{self, Duration};

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
    game_state: Arc<Mutex<Game>>,
    game_start_timer: Option<Instant>,
}

impl Server {
    pub fn new<S: Into<String>>(host: S, port: u16) -> Server {
        Server {
            host: host.into(),
            port,
            min_players: 1,
            max_players: 10,
            game_state: Arc::new(Mutex::new(Game::new())),
            game_start_timer: None,
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

        // Create a timer check task
        let game_state_timer = game_state.clone();
        let socket_timer = socket.clone();
        let min_players = self.min_players;

        tokio::spawn(async move {
            let mut interval = time::interval(Duration::from_secs(1));
            loop {
                interval.tick().await;

                let mut state = game_state_timer.lock().await;

                // Check if we have enough players and game is in waiting state
                if state.state == super::game::GameState::Waiting {
                    let player_count = state.players.len() as u8;

                    // Set start timer if we have enough players and timer isn't set yet
                    if player_count >= min_players {
                        if state.game_start_time.is_none() {
                            state.game_start_time = Some(Instant::now());
                            log::info!("Minimum player count reached ({}/{}). Starting 5 second countdown!",
                                      player_count, min_players);
                        }
                    } else if state.game_start_time.is_some() {
                        // Cancel timer if player count drops below minimum
                        state.game_start_time = None;
                        log::info!("Player count dropped below minimum. Cancelling countdown.");
                    }

                    // Check if timer has elapsed
                    if let Some(start_time) = state.game_start_time {
                        let elapsed = start_time.elapsed();
                        if elapsed >= Duration::from_secs(5) {
                            // 5 seconds countdown
                            state.state = super::game::GameState::InProgress;
                            log::info!("Game starting after 5 second countdown!");

                            // Send game start message to all players
                            let message = ServerMessage::GameStart {
                                maze_level: state.maze_level,
                            };
                            if let Err(e) = Self::broadcast_message_static(
                                &socket_timer,
                                message,
                                &state.players,
                            )
                            .await
                            {
                                log::error!("Failed to broadcast game start: {}", e);
                            }
                        }
                    }
                }
            }
        });

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
                ClientMessage::ShotPlayer { player_username } => {
                    self.handle_shoot(game_state.clone(), socket.clone(), addr, player_username)
                        .await?;
                }
            }
        }
    }
}

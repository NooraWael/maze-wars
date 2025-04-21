use shared::{server::ServerMessage, Player, Position, Rotation, Weapon};
use std::{net::SocketAddr, sync::Arc, time::Instant};
use tokio::net::UdpSocket;
use tokio::sync::Mutex;

use super::{
    game::{Game, GameState},
    Server,
};

impl Server {
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
    pub async fn handle_join_game(
        &self,
        game_state: Arc<Mutex<Game>>,
        socket: Arc<UdpSocket>,
        addr: SocketAddr,
        username: String,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut state = game_state.lock().await;

        // Check if the player with the same name already exists
        if state.players.values().any(|p| p.username == username) {
            let error_message = ServerMessage::JoinGameError {
                message: "Username already taken".to_string(),
            };
            self.send_message(&socket, error_message, &addr).await?;
            return Ok(());
        }

        // Check if the player limit is reached
        if state.players.len() >= self.max_players as usize {
            let error_message = ServerMessage::JoinGameError {
                message: "Server is full".to_string(),
            };
            self.send_message(&socket, error_message, &addr).await?;
            return Ok(());
        }

        let player = Player::new(
            username.clone(),
            Default::default(),
            Player::DEFAULT_HEIGHT,
            Default::default(),
            100,
            Weapon::pistol(),
        );
        state.players.insert(addr, player);
        log::info!("New player connection: {} from {}", username, addr);

        // Start timer if we have enough players but game hasn't started yet
        let player_count = state.players.len();
        if player_count >= self.min_players as usize
            && player_count <= self.max_players as usize
            && state.state == GameState::Waiting
            && state.game_start_time.is_none()
        {
            log::info!("Minimum player count reached, starting 5 seconds timer until game begins");
            state.game_start_time = Some(Instant::now());

            // Inform players about the timer
            let info_message = ServerMessage::GameStart;
            self.broadcast_message(&socket, info_message, &state.players)
                .await?;
        }

        let response = ServerMessage::PlayersInLobby {
            player_count: state.players.len() as u32,
            players: state.players.values().map(|p| p.username.clone()).collect(),
        };

        self.broadcast_message(&socket, response, &state.players)
            .await?;
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
    pub async fn handle_move(
        &self,
        game_state: Arc<Mutex<Game>>,
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
    pub async fn handle_shoot(
        &self,
        game_state: Arc<Mutex<Game>>,
        socket: Arc<UdpSocket>,
        addr: SocketAddr,
        player_to_shoot: String,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut state = game_state.lock().await;
        let shooter_username = match state.players.get(&addr) {
            Some(p) => p.username.clone(),
            None => {
                log::warn!("Shooter not found for address {}", addr);
                return Ok(());
            }
        };

        // Find the address of the player to shoot by username
        let target_addr = match state
            .players
            .iter()
            .find(|(_, p)| p.username == player_to_shoot)
        {
            Some((addr, _)) => *addr,
            None => {
                log::warn!("Player to shoot not found: {}", player_to_shoot);
                return Ok(());
            }
        };

        // Extract the necessary information and update the player in a separate scope
        let (target_username, target_health) = {
            let target_player = state.players.get_mut(&target_addr).unwrap();
            // Reduce health by 10, saturating at 0
            target_player.health = target_player.health.saturating_sub(10);
            (target_player.username.clone(), target_player.health)
        };

        log::debug!(
            "Player {} fired at {} (new health: {})",
            shooter_username,
            target_username,
            target_health
        );

        // Emit HealthUpdate to all players
        let health_update = ServerMessage::HealthUpdate {
            player_id: target_username.clone(),
            health: target_health,
        };
        self.broadcast_message(&socket, health_update, &state.players)
            .await?;

        // If health reaches 0, emit PlayerDeath to all players
        if target_health == 0 {
            let death_message = ServerMessage::PlayerDeath {
                player_id: target_username,
                killer_id: Some(shooter_username),
            };
            self.broadcast_message(&socket, death_message, &state.players)
                .await?;
        }
        // If one player is left alive, emit GameOver
        // Count alive players (health > 0)
        let alive_players: Vec<_> = state.players.values().filter(|p| p.health > 0).collect();

        // If only one player is alive, emit GameOver
        if alive_players.len() == 1 {
            let winner = alive_players[0].username.clone();
            let game_over_message = ServerMessage::GameOver { winner };
            self.broadcast_message(&socket, game_over_message, &state.players)
                .await?;
        }

        Ok(())
    }
}

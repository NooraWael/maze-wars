use std::{net::SocketAddr, sync::Arc};
use tokio::net::UdpSocket;
use tokio::sync::Mutex;

use crate::player::Player;
use crate::position::Position;
use crate::rotation::Rotation;
use crate::weapon::Weapon;

use super::{game_state::GameState, Server, ServerMessage};

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
        game_state: Arc<Mutex<GameState>>,
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
    pub async fn handle_shoot(
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
}

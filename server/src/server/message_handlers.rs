use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::UdpSocket;

use super::Server;
use super::ServerMessage;
use crate::player::Player;

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
    pub async fn send_message(
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
    pub async fn broadcast_message(
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
}

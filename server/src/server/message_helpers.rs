use shared::server::ServerMessage;
use shared::Player;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::UdpSocket;

use super::Server;

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

    /// Static version of broadcast_message that can be used from timer tasks
    pub async fn broadcast_message_static(
        socket: &Arc<UdpSocket>,
        message: ServerMessage,
        players: &std::collections::HashMap<std::net::SocketAddr, shared::Player>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let message_json = serde_json::to_string(&message)?;
        for addr in players.keys() {
            if let Err(e) = socket.send_to(message_json.as_bytes(), addr).await {
                log::warn!("Failed to send message to {}: {}", addr, e);
            }
        }
        Ok(())
    }
}

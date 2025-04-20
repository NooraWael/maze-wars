use anyhow::Result;
use shared::server::{ClientMessage, ServerMessage};
use std::net::{ToSocketAddrs, UdpSocket};

pub struct NetworkClient {
    socket: UdpSocket,
    server_addr: String,
}

impl NetworkClient {
    pub fn new(bind_addr: &str, server_addr: &str) -> Result<Self> {
        let socket = UdpSocket::bind(bind_addr)?;
        socket.set_nonblocking(true)?;
        Ok(Self {
            socket,
            server_addr: server_addr.to_string(),
        })
    }

    pub fn send(&self, msg: &ClientMessage) -> Result<()> {
        let json = serde_json::to_string(msg)?;

        // Resolve to a valid socket address
        let mut addrs_iter = self.server_addr.to_socket_addrs()?;
        if let Some(addr) = addrs_iter.next() {
            self.socket.send_to(json.as_bytes(), addr)?;
        } else {
            return Err(anyhow::anyhow!("Could not resolve server address"));
        }

        Ok(())
    }

    pub fn try_receive(&self) -> Option<ServerMessage> {
        let mut buf = [0u8; 1024];
        match self.socket.recv_from(&mut buf) {
            Ok((len, _addr)) => {
                let msg = String::from_utf8_lossy(&buf[..len]);
                serde_json::from_str(&msg).ok()
            }
            Err(_) => None,
        }
    }
}

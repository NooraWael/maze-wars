use shared::Player;

use std::{collections::HashMap, net::SocketAddr};

#[derive(Debug)]
pub struct GameState {
    pub players: HashMap<SocketAddr, Player>,
}

impl GameState {
    pub fn new() -> Self {
        GameState {
            players: HashMap::new(),
        }
    }
}

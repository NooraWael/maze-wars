use std::{collections::HashMap, net::SocketAddr};

use crate::player::Player;

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

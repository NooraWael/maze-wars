use shared::Player;

use std::{collections::HashMap, net::SocketAddr};
#[derive(Debug)]
pub enum GameState {
    Waiting,
    InProgress,
    Finished,
}

#[derive(Debug)]
pub struct Game {
    pub players: HashMap<SocketAddr, Player>,
    pub state: GameState,
}

impl Game {
    pub fn new() -> Self {
        Game {
            players: HashMap::new(),
            state: GameState::Waiting,
        }
    }
}

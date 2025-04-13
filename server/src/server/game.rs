use shared::Player;
use std::{collections::HashMap, net::SocketAddr, time::Instant};

#[derive(Debug, PartialEq)]
pub enum GameState {
    Waiting,
    InProgress,
    Finished,
}

#[derive(Debug)]
pub struct Game {
    pub players: HashMap<SocketAddr, Player>,
    pub state: GameState,
    pub game_start_time: Option<Instant>,
}

impl Game {
    pub fn new() -> Self {
        Game {
            players: HashMap::new(),
            state: GameState::Waiting,
            game_start_time: None,
        }
    }
}

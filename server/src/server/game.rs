use shared::Player;
use std::{collections::HashMap, net::SocketAddr, time::Instant};
use rand::Rng;

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
    pub maze_level: u8,
}

impl Game {
    pub fn new() -> Self {
        let maze_level = (rand::thread_rng().gen_range(0..3) + 1) as u8;
        Game {
            players: HashMap::new(),
            state: GameState::Waiting,
            game_start_time: None,
            maze_level,
        }
    }
}

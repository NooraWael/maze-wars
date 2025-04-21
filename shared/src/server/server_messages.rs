use serde::{Deserialize, Serialize};

use crate::{rotation::Rotation, Position};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type", content = "data")]
pub enum ServerMessage {
    Error {
        message: String,
    },
    JoinGameError {
        message: String,
    },
    PlayersInLobby {
        player_count: u32,
        players: Vec<String>,
    },
    GameStart {
        maze_level: u8, 
    },
    PlayerMove {
        player_id: String,
        position: Position,
        rotation: Rotation,
        yield_control: f32,
    },

    PlayerDeath {
        player_id: String,
        killer_id: Option<String>,
    },
    PlayerSpawn {
        player_id: String,
        position: Position,
    },
    HealthUpdate {
        player_id: String,
        health: u32,
    },
    GameOver {
        winner: String,
    },
}

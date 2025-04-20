use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type", content = "data")]
pub enum ClientMessage {
    JoinGame {
        username: String,
    },
    Move {
        position: Position,
        rotation: Rotation,
        yield_control: f32,
    },
    ShotPlayer {
        player_username: String,
    },
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type", content = "data")]
pub enum ServerMessage {
    JoinGameError {
        message: String,
    },
    PlayersInLobby {
        player_count: u32,
        players: Vec<String>,
    },
    GameStart,
    PlayerMove {
        player_id: String,
        position: Position,
        rotation: Rotation,
        yield_control: f32,
    },
    Error {
        message: String,
    },
}

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Copy)]
pub struct Position {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct Rotation {
    pub yaw: f32,
    pub pitch: f32,
    #[serde(default)]
    pub roll: f32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Player {
    pub username: String,
    pub position: Position,
}

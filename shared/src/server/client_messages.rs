use serde::{Deserialize, Serialize};

use crate::{rotation::Rotation, Position};

#[derive(Debug, Serialize, Deserialize)]
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

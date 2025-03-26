use serde::{Deserialize, Serialize};

use crate::{position::Position, rotation::Rotation};

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
    Shoot {
        position: Position,
        direction: Rotation,
        weapon_type: String,
    },
}

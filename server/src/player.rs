use serde::{Deserialize, Serialize};

use crate::{position::Position, rotation::Rotation, weapon::Weapon};

#[derive(Debug, Serialize, Deserialize, Clone)]
/// Represents a connected player with their current state
///
/// # Fields
/// - `username`: Player's display name
/// - `position`: Current 3D coordinates
/// - `rotation`: Current orientation
/// - `health`: Health points (0-100)
/// - `weapon`: Equipped weapon stats
pub struct Player {
    pub username: String,
    pub position: Position,
    pub rotation: Rotation,
    pub health: u32,
    pub weapon: Weapon,
}

impl Player {
    /// Creates a new `Player` instance
    ///
    /// # Arguments
    /// - `username`: The player's display name
    /// - `position`: The initial position of the player
    /// - `rotation`: The initial rotation of the player
    /// - `health`: The initial health of the player
    /// - `weapon`: The initial weapon of the player
    ///
    /// # Returns
    /// A new `Player` instance
    pub fn new(
        username: String,
        position: Position,
        rotation: Rotation,
        health: u32,
        weapon: Weapon,
    ) -> Self {
        Self {
            username,
            position,
            rotation,
            health,
            weapon,
        }
    }
}

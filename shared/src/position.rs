use bevy::math::Vec3;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
/// Represents a 3D position in game world
///
/// # Examples
/// ```rust
/// let pos = Position::new(10.0, 5.0, 2.5);
/// ```
pub struct Position {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Position {
    /// Creates a new `Position` instance.
    ///
    /// # Arguments
    /// * `x` - The x-coordinate.
    /// * `y` - The y-coordinate.
    /// * `z` - The z-coordinate.
    ///
    /// # Examples
    /// ```rust
    /// let pos = Position::new(10.0, 5.0, 2.5);
    /// ```
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }
}

impl Default for Position {
    /// Provides a default `Position` instance with all coordinates set to `0.0`.
    fn default() -> Self {
        Self::new(0., 0., 0.)
    }
}

impl From<Vec3> for Position {
    fn from(vec: Vec3) -> Self {
        Self {
            x: vec.x,
            y: vec.y,
            z: vec.z,
        }
    }
}

impl Into<Vec3> for Position {
    fn into(self) -> Vec3 {
        Vec3::new(self.x, self.y, self.z)
    }
}

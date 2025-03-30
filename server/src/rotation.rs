use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
/// Represents player's rotation/orientation in 3D space
///
/// # Examples
/// ```rust
/// let rot = Rotation { pitch: 90.0, yaw: 45.0, roll: 0.0 };
/// ```
pub struct Rotation {
    pitch: f32,
    yaw: f32,
    roll: f32,
}

impl Rotation {
    /// Creates a new `Rotation` instance with the given pitch, yaw, and roll.
    pub fn new(pitch: f32, yaw: f32, roll: f32) -> Self {
        Self { pitch, yaw, roll }
    }
}

impl Default for Rotation {
    /// Returns a default `Rotation` with all values set to 0.0.
    fn default() -> Self {
        Self::new(0.0, 0.0, 0.0)
    }
}

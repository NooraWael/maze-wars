use bevy::math::{EulerRot, Quat};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
/// Represents player's rotation/orientation in 3D space
///
/// # Examples
/// ```rust
/// let rot = Rotation { pitch: 90.0, yaw: 45.0, roll: 0.0 };
/// ```
pub struct Rotation {
    pub pitch: f32,
    pub yaw: f32,
    pub roll: f32,
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

impl From<Quat> for Rotation {
    fn from(quat: Quat) -> Self {
        let (yaw, pitch, roll) = quat.to_euler(EulerRot::YXZ);
        Self { pitch, yaw, roll }
    }
}

impl Into<Quat> for Rotation {
    fn into(self) -> Quat {
        Quat::from_euler(EulerRot::YXZ, self.yaw, self.pitch, self.roll)
    }
}

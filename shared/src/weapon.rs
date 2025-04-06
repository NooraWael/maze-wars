use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
/// Defines weapon characteristics for a player
///
/// # Examples
/// ```rust
/// let weapon = Weapon::pistol();
/// ```
pub struct Weapon {
    pub name: String,
    pub damage: u32,
    pub fire_rate: f32,
    pub ammo_count: u32,
    pub range: f32,
}

impl Weapon {
    /// Returns a predefined pistol weapon
    pub fn pistol() -> Self {
        Weapon {
            name: String::from("Pistol"),
            damage: 25,
            fire_rate: 1.5,
            ammo_count: 12,
            range: 30.0,
        }
    }

    // /// Returns a predefined rifle weapon
    // pub fn rifle() -> Self {
    //     Weapon {
    //         name: String::from("Rifle"),
    //         damage: 40,
    //         fire_rate: 3.0,
    //         ammo_count: 30,
    //         range: 60.0,
    //     }
    // }

    // /// Returns a predefined sniper weapon
    // pub fn sniper() -> Self {
    //     Weapon {
    //         name: String::from("Sniper"),
    //         damage: 90,
    //         fire_rate: 0.8,
    //         ammo_count: 5,
    //         range: 100.0,
    //     }
    // }
}

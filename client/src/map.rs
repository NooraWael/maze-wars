use rand::seq::SliceRandom;
use rand::thread_rng;

pub const MAZE_WIDTH: usize = 20;
pub const MAZE_HEIGHT: usize = 15;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Tile {
    Wall,
    Floor,
}

pub type MazeMap = [[Tile; MAZE_WIDTH]; MAZE_HEIGHT];
pub type SpawnPoints = Vec<(f32, f32)>;

pub struct MazeLevel {
    pub map: MazeMap,
    pub spawns: SpawnPoints,
}

// LEVEL 1 - Easy: Open layout with some strategic walls
pub fn level_1() -> MazeLevel {
    let mut map = [[Tile::Floor; MAZE_WIDTH]; MAZE_HEIGHT];
    
    // Add outer walls
    for y in 0..MAZE_HEIGHT {
        for x in 0..MAZE_WIDTH {
            if x == 0 || y == 0 || x == MAZE_WIDTH - 1 || y == MAZE_HEIGHT - 1 {
                map[y][x] = Tile::Wall;
            }
        }
    }
    
    // Add strategically placed obstacles rather than linear walls
    
    // Create some "room" dividers in the center
    for x in 8..12 {
        map[7][x] = Tile::Wall;
    }
    
    // Left side pillar obstacles
    map[3][4] = Tile::Wall;
    map[3][5] = Tile::Wall;
    map[4][4] = Tile::Wall;
    map[4][5] = Tile::Wall;
    
    map[3][14] = Tile::Wall;
    map[3][15] = Tile::Wall;
    map[4][14] = Tile::Wall;
    map[4][15] = Tile::Wall;
    
    map[10][4] = Tile::Wall;
    map[10][5] = Tile::Wall;
    map[11][4] = Tile::Wall;
    map[11][5] = Tile::Wall;
    
    map[10][14] = Tile::Wall;
    map[10][15] = Tile::Wall;
    map[11][14] = Tile::Wall;
    map[11][15] = Tile::Wall;
    
    // Add some single wall obstacles for cover
    map[3][9] = Tile::Wall;
    map[11][9] = Tile::Wall;
    map[7][3] = Tile::Wall;
    map[7][16] = Tile::Wall;
    
    // Make sure the central divider has openings
    map[7][8] = Tile::Floor;
    map[7][11] = Tile::Floor;
    
    // First 4 spawn points in different corners/directions
    // Then the rest well distributed
    let spawns = vec![
        (2.5, 2.5),     // Top left corner
        (17.5, 2.5),    // Top right corner
        (2.5, 12.5),    // Bottom left corner
        (17.5, 12.5),   // Bottom right corner
        (9.5, 2.5),     // Top center
        (9.5, 12.5),    // Bottom center
        (2.5, 7.5),     // Middle left
        (17.5, 7.5),    // Middle right
        (6.5, 5.5),     // Additional strategic positions
        (13.5, 9.5),
    ];
    
    MazeLevel { map, spawns }
}

// LEVEL 2 - Medium: Simplified layout with strategic cover
pub fn level_2() -> MazeLevel {
    let mut map = [[Tile::Floor; MAZE_WIDTH]; MAZE_HEIGHT];
    
    // Add outer walls
    for y in 0..MAZE_HEIGHT {
        for x in 0..MAZE_WIDTH {
            if x == 0 || y == 0 || x == MAZE_WIDTH - 1 || y == MAZE_HEIGHT - 1 {
                map[y][x] = Tile::Wall;
            }
        }
    }
    
    // Add symmetrical T-shaped obstacles for cover
    
    // Top T obstacles
    map[3][5] = Tile::Wall;
    map[3][6] = Tile::Wall;
    map[3][7] = Tile::Wall;
    map[4][6] = Tile::Wall;
    map[5][6] = Tile::Wall;
    
    map[3][12] = Tile::Wall;
    map[3][13] = Tile::Wall;
    map[3][14] = Tile::Wall;
    map[4][13] = Tile::Wall;
    map[5][13] = Tile::Wall;
    
    // Bottom T obstacles
    map[11][5] = Tile::Wall;
    map[11][6] = Tile::Wall;
    map[11][7] = Tile::Wall;
    map[10][6] = Tile::Wall;
    map[9][6] = Tile::Wall;
    
    map[11][12] = Tile::Wall;
    map[11][13] = Tile::Wall;
    map[11][14] = Tile::Wall;
    map[10][13] = Tile::Wall;
    map[9][13] = Tile::Wall;
    
    // Center dividers - just single walls to create flow
    for y in 6..9 {
        map[y][9] = Tile::Wall;
    }
    
    // Some additional blocks for cover
    map[2][2] = Tile::Wall;
    map[2][17] = Tile::Wall;
    map[12][2] = Tile::Wall;
    map[12][17] = Tile::Wall;
    
    map[7][2] = Tile::Wall;
    map[7][3] = Tile::Wall;
    
    map[7][16] = Tile::Wall;
    map[7][17] = Tile::Wall;
    
    // Create several L shaped covers in the center
    map[6][4] = Tile::Wall;
    map[7][4] = Tile::Wall;
    map[7][5] = Tile::Wall;
    
    map[6][15] = Tile::Wall;
    map[7][15] = Tile::Wall;
    map[7][14] = Tile::Wall;
    
    // First 4 spawn points in different corners/directions
    // Then the rest distributed throughout the map
    let spawns = vec![
        (2.5, 2.5),     // Top left corner
        (17.5, 2.5),    // Top right corner
        (2.5, 12.5),    // Bottom left corner
        (17.5, 12.5),   // Bottom right corner
        (9.5, 2.5),     // Top center
        (9.5, 12.5),    // Bottom center
        (3.5, 7.5),     // Middle left
        (16.5, 7.5),    // Middle right
        (9.5, 7.5),     // Center
        (12.5, 4.5),    // Upper right quadrant
    ];
    
    MazeLevel { map, spawns }
}

// LEVEL 3 - Hard: Simple but strategic with more cover
pub fn level_3() -> MazeLevel {
    let mut map = [[Tile::Floor; MAZE_WIDTH]; MAZE_HEIGHT];
    
    // Add outer walls
    for y in 0..MAZE_HEIGHT {
        for x in 0..MAZE_WIDTH {
            if x == 0 || y == 0 || x == MAZE_WIDTH - 1 || y == MAZE_HEIGHT - 1 {
                map[y][x] = Tile::Wall;
            }
        }
    }
    
    // Add a plus-shaped obstacle in the center
    for x in 8..12 {
        map[7][x] = Tile::Wall;
    }
    
    for y in 5..10 {
        map[y][10] = Tile::Wall;
    }
    
    // Add scattered 2x2 and 2x1 obstacle blocks for cover
    
    // Upper left quadrant
    map[3][3] = Tile::Wall;
    map[3][4] = Tile::Wall;
    map[4][3] = Tile::Wall;
    map[4][4] = Tile::Wall;
    
    map[3][7] = Tile::Wall;
    map[4][7] = Tile::Wall;
    
    // Upper right quadrant
    map[3][15] = Tile::Wall;
    map[3][16] = Tile::Wall;
    map[4][15] = Tile::Wall;
    map[4][16] = Tile::Wall;
    
    map[3][12] = Tile::Wall;
    map[4][12] = Tile::Wall;
    
    // Lower left quadrant
    map[10][3] = Tile::Wall;
    map[10][4] = Tile::Wall;
    map[11][3] = Tile::Wall;
    map[11][4] = Tile::Wall;
    
    map[10][7] = Tile::Wall;
    map[11][7] = Tile::Wall;
    
    // Lower right quadrant
    map[10][15] = Tile::Wall;
    map[10][16] = Tile::Wall;
    map[11][15] = Tile::Wall;
    map[11][16] = Tile::Wall;
    
    map[10][12] = Tile::Wall;
    map[11][12] = Tile::Wall;
    
    // Add single blocks in scattered positions for easy cover
    map[2][10] = Tile::Wall;
    map[12][10] = Tile::Wall;
    map[7][2] = Tile::Wall;
    map[7][17] = Tile::Wall;
    
    map[5][5] = Tile::Wall;
    map[5][14] = Tile::Wall;
    map[9][5] = Tile::Wall;
    map[9][14] = Tile::Wall;
    
    // First 4 spawn points in different corners/directions
    // Then the rest well distributed
    let spawns = vec![
        (2.5, 2.5),     // Top left corner
        (17.5, 2.5),    // Top right corner
        (2.5, 12.5),    // Bottom left corner
        (17.5, 12.5),   // Bottom right corner
        (9.5, 2.5),     // Top center
        (9.5, 12.5),    // Bottom center
        (2.5, 7.5),     // Middle left
        (17.5, 7.5),    // Middle right
        (6.5, 5.5),     // Near top-left quadrant
        (13.5, 9.5),    // Near bottom-right quadrant
    ];
    
    MazeLevel { map, spawns }
}

/// Generates a specific maze level by index (1, 2, or 3)
pub fn get_maze_level(level_index: u8) -> MazeLevel {
    match level_index {
        1 => level_1(),
        2 => level_2(),
        3 => level_3(),
        _ => level_1(), // Default to level 1 for any invalid index
    }
}

/// Randomly chooses one of the levels
pub fn generate_maze_level() -> MazeLevel {
    let mut levels = vec![level_1(), level_2(), level_3()];
    let mut rng = thread_rng();
    levels.shuffle(&mut rng);
    levels.remove(0)
}
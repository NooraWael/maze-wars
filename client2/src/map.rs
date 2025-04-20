pub const MAZE_WIDTH: usize = 20;
pub const MAZE_HEIGHT: usize = 15;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Tile {
    Wall,
    Floor,
}

pub type MazeMap = [[Tile; MAZE_WIDTH]; MAZE_HEIGHT];

pub fn generate_maze() -> MazeMap {
    let mut map = [[Tile::Floor; MAZE_WIDTH]; MAZE_HEIGHT];

    for y in 0..MAZE_HEIGHT {
        for x in 0..MAZE_WIDTH {
            if x == 0 || y == 0 || x == MAZE_WIDTH - 1 || y == MAZE_HEIGHT - 1 || (x % 2 == 0 && y % 2 == 0) {
                map[y][x] = Tile::Wall;
            }
        }
    }

    map
}

mod map;
mod net;

use crate::map::{generate_maze, Tile, MAZE_HEIGHT, MAZE_WIDTH};
use crate::net::NetworkClient;
use std::io::{self, Write};
use std::time::{Duration, Instant};

use sdl2::{
    event::Event, keyboard::Keycode, pixels::Color, rect::Rect, render::Canvas, video::Window,
};
use shared::server::{ClientMessage, ServerMessage};
use shared::{Position, Rotation};
use std::collections::HashMap;




const SCREEN_WIDTH: u32 = 800;
const SCREEN_HEIGHT: u32 = 600;
const MINIMAP_TILE_SIZE: u32 = 20;
const FOV: f32 = std::f32::consts::FRAC_PI_4; // 45 degrees
const RAY_DISTANCE: f32 = 10.0;

#[derive(Debug, Clone, Copy)]
struct Player3D {
    x: f32,
    y: f32,
    angle: f32,
}

fn prompt(text: &str) -> String {
    print!("{}", text);
    io::stdout().flush().unwrap();
    let mut buf = String::new();
    io::stdin().read_line(&mut buf).unwrap();
    buf.trim().to_string()
}

fn cast_ray(
    maze: &[[Tile; MAZE_WIDTH]; MAZE_HEIGHT],
    player: &Player3D,
    angle: f32,
) -> Option<(f32, f32)> {
    let mut x = player.x;
    let mut y = player.y;
    let dx = angle.cos();
    let dy = angle.sin();

    for _ in 0..(RAY_DISTANCE * 10.0) as usize {
        x += dx * 0.1;
        y += dy * 0.1;

        let grid_x = x as usize;
        let grid_y = y as usize;

        if grid_x >= MAZE_WIDTH || grid_y >= MAZE_HEIGHT {
            return None;
        }

        if maze[grid_y][grid_x] == Tile::Wall {
            return Some((x, y));
        }
    }
    None
}

fn render_first_person_view(
    canvas: &mut Canvas<Window>,
    maze: &[[Tile; MAZE_WIDTH]; MAZE_HEIGHT],
    player: &Player3D,
) {
    canvas.set_draw_color(Color::RGB(135, 206, 235)); // Sky blue
    canvas
        .fill_rect(Rect::new(0, 0, SCREEN_WIDTH, SCREEN_HEIGHT / 2))
        .unwrap();

    canvas.set_draw_color(Color::RGB(50, 150, 50)); // Ground green
    canvas
        .fill_rect(Rect::new(
            0,
            SCREEN_HEIGHT as i32 / 2,
            SCREEN_WIDTH,
            SCREEN_HEIGHT / 2,
        ))
        .unwrap();

    // Cast rays
    for x in 0..SCREEN_WIDTH {
        // Calculate ray angle
        let ray_angle = player.angle - FOV / 2.0 + (x as f32 / SCREEN_WIDTH as f32) * FOV;

        if let Some((hit_x, hit_y)) = cast_ray(maze, player, ray_angle) {
            // Calculate distance to wall
            let distance = ((hit_x - player.x).powi(2) + (hit_y - player.y).powi(2)).sqrt();

            // Calculate wall height based on distance
            let wall_height = (SCREEN_HEIGHT as f32 / distance).min(SCREEN_HEIGHT as f32);

            // Draw wall slice
            let wall_top = (SCREEN_HEIGHT as f32 - wall_height) / 2.0;
            canvas.set_draw_color(Color::RGB(100, 100, 100)); // Gray wall color
            canvas
                .fill_rect(Rect::new(x as i32, wall_top as i32, 1, wall_height as u32))
                .unwrap();
        }
    }
}

fn render_minimap(
    canvas: &mut Canvas<Window>,
    maze: &[[Tile; MAZE_WIDTH]; MAZE_HEIGHT],
    player: &Player3D,
    other_players: &HashMap<String, (Position, Rotation)>
) {
    for (y, row) in maze.iter().enumerate() {
        for (x, tile) in row.iter().enumerate() {
            let color = match tile {
                Tile::Wall => Color::RGB(80, 80, 80),
                Tile::Floor => Color::RGB(200, 200, 200),
            };

            canvas.set_draw_color(color);
            let _ = canvas.fill_rect(Rect::new(
                (x * MINIMAP_TILE_SIZE as usize) as i32,
                (y * MINIMAP_TILE_SIZE as usize) as i32,
                MINIMAP_TILE_SIZE,
                MINIMAP_TILE_SIZE,
            ));
        }
    }

    // Player marker
    canvas.set_draw_color(Color::RGB(255, 0, 0));
    let _ = canvas.fill_rect(Rect::new(
        (player.x as usize * MINIMAP_TILE_SIZE as usize) as i32,
        (player.y as usize * MINIMAP_TILE_SIZE as usize) as i32,
        MINIMAP_TILE_SIZE,
        MINIMAP_TILE_SIZE,
    ));

    // Render other players
canvas.set_draw_color(Color::RGB(0, 0, 255)); // Blue for other players
for (_name, (pos, _rot)) in other_players.iter() {
    let _ = canvas.fill_rect(Rect::new(
        (pos.x as usize * MINIMAP_TILE_SIZE as usize) as i32,
        (pos.y as usize * MINIMAP_TILE_SIZE as usize) as i32,
        MINIMAP_TILE_SIZE,
        MINIMAP_TILE_SIZE,
    ));
}

}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    let username = prompt("Enter Name: ");
    let server_addr = prompt("Enter IP Address (example 127.0.0.1:2025): ");
    let mut other_players: HashMap<String, (Position, Rotation)> = HashMap::new();

    let client = NetworkClient::new("0.0.0.0:0", &server_addr)?;

    client.send(&ClientMessage::JoinGame { username: username.clone() })?;

    println!("Waiting to join lobby...");

    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;
    let window = video_subsystem
        .window("Multiplayer FPS", SCREEN_WIDTH, SCREEN_HEIGHT)
        .position_centered()
        .build()?;

    let mut canvas = window.into_canvas().build()?;
    let mut event_pump = sdl_context.event_pump()?;

    let mut running = true;
    let mut last_frame = Instant::now();
    let mut game_started = false;

    let maze_map = generate_maze();
    let mut player = Player3D {
        x: 1.5,
        y: 1.5,
        angle: 0.0,
    };

    let mut last_sent_position = Position::default();
    let mut last_sent_rotation = Rotation {
        yaw: 0.0,
        pitch: 0.0,
        roll: 0.0,
    };

    while running {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => {
                    running = false;
                }

                Event::KeyDown {
                    keycode: Some(Keycode::W),
                    ..
                } if game_started => {
                    let new_x = player.x + player.angle.cos() * 0.1;
                    let new_y = player.y + player.angle.sin() * 0.1;
                    let grid_x = new_x as usize;
                    let grid_y = new_y as usize;

                    if grid_x < MAZE_WIDTH
                        && grid_y < MAZE_HEIGHT
                        && maze_map[grid_y][grid_x] != Tile::Wall
                    {
                        player.x = new_x;
                        player.y = new_y;
                    }
                }
                Event::KeyDown {
                    keycode: Some(Keycode::S),
                    ..
                } if game_started => {
                    let new_x = player.x - player.angle.cos() * 0.1;
                    let new_y = player.y - player.angle.sin() * 0.1;
                    let grid_x = new_x as usize;
                    let grid_y = new_y as usize;

                    if grid_x < MAZE_WIDTH
                        && grid_y < MAZE_HEIGHT
                        && maze_map[grid_y][grid_x] != Tile::Wall
                    {
                        player.x = new_x;
                        player.y = new_y;
                    }
                }
                Event::KeyDown {
                    keycode: Some(Keycode::A),
                    ..
                } if game_started => {
                    player.angle -= 0.1;
                }
                Event::KeyDown {
                    keycode: Some(Keycode::D),
                    ..
                } if game_started => {
                    player.angle += 0.1;
                }

                _ => {}
            }
        }

        // Update game state from server
        if let Some(msg) = client.try_receive() {
            match msg {
                ServerMessage::GameStart => {
                    println!("Game starting!");
                    game_started = true;
                }
                ServerMessage::PlayersInLobby {
                    player_count,
                    players,
                } => {
                    println!("{} players connected: {:?}", player_count, players);
                }
                ServerMessage::JoinGameError { message } => {
                    println!("Error: {}", message);
                    return Ok(());
                }
                ServerMessage::PlayerMove {
                    player_id,
                    position,
                    rotation,
                    ..
                } => {
                    if player_id != username {
                        other_players.insert(player_id, (position, rotation));
                    }
                }
                _ => {}
            }
        }

        // Example movement send (dummy)
        let position = Position {
            x: player.x,
            y: player.y,
            z: 0.0,
        };
        let rotation = Rotation {
            yaw: player.angle.to_degrees(),
            pitch: 0.0,
            roll: 0.0,
        };

        // Only send move if something changed
        if position != last_sent_position || rotation != last_sent_rotation {
            client.send(&ClientMessage::Move {
                position,
                rotation,
                yield_control: 0.5,
            })?;

            last_sent_position = position;
            last_sent_rotation = rotation;
        }

        canvas.set_draw_color(Color::RGB(30, 30, 30));
        canvas.clear();

        if game_started {
            render_first_person_view(&mut canvas, &maze_map, &player);
            render_minimap(&mut canvas, &maze_map, &player, &other_players);

        }

        canvas.present();

        // Keep 60 FPS
        let elapsed = last_frame.elapsed();
        if elapsed < Duration::from_millis(16) {
            std::thread::sleep(Duration::from_millis(16) - elapsed);
        }
        last_frame = Instant::now();
    }

    Ok(())
}

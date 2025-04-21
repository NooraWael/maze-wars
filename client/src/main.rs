mod map;
mod net;

use crate::map::{generate_maze, Tile, MAZE_HEIGHT, MAZE_WIDTH};
use crate::net::NetworkClient;
use sdl2::{
    event::Event, keyboard::Keycode, pixels::Color, rect::Rect, render::Canvas, video::Window,
};
use shared::server::{ClientMessage, ServerMessage};
use shared::{Position, Rotation};
use std::collections::HashMap;
use std::io::{self, Write};
use std::time::{Duration, Instant};

const SCREEN_WIDTH: u32 = 800;
const SCREEN_HEIGHT: u32 = 600;
const MINIMAP_TILE_SIZE: u32 = 10;
const FOV: f32 = std::f32::consts::FRAC_PI_4;
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

fn has_line_of_sight(
    maze: &[[Tile; MAZE_WIDTH]; MAZE_HEIGHT],
    from: (f32, f32),
    to: (f32, f32),
) -> bool {
    let dx = to.0 - from.0;
    let dy = to.1 - from.1;
    let distance = (dx * dx + dy * dy).sqrt();

    let steps = (distance / 0.05).ceil() as usize;
    for i in 0..steps {
        let t = i as f32 / steps as f32;
        let x = from.0 + dx * t;
        let y = from.1 + dy * t;

        let gx = x as usize;
        let gy = y as usize;

        if gx >= MAZE_WIDTH || gy >= MAZE_HEIGHT {
            return false;
        }

        if maze[gy][gx] == Tile::Wall {
            return false;
        }
    }

    true
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
    other_players: &HashMap<String, (Position, Rotation)>,
) {
    canvas.set_draw_color(Color::RGB(135, 206, 235));
    canvas
        .fill_rect(Rect::new(0, 0, SCREEN_WIDTH, SCREEN_HEIGHT / 2))
        .unwrap();

    canvas.set_draw_color(Color::RGB(50, 150, 50));
    canvas
        .fill_rect(Rect::new(
            0,
            SCREEN_HEIGHT as i32 / 2,
            SCREEN_WIDTH,
            SCREEN_HEIGHT / 2,
        ))
        .unwrap();

    for x in 0..SCREEN_WIDTH {
        let ray_angle = player.angle - FOV / 2.0 + (x as f32 / SCREEN_WIDTH as f32) * FOV;

        if let Some((hit_x, hit_y)) = cast_ray(maze, player, ray_angle) {
            let distance = ((hit_x - player.x).powi(2) + (hit_y - player.y).powi(2)).sqrt();
            let wall_height = (SCREEN_HEIGHT as f32 / distance).min(SCREEN_HEIGHT as f32);
            let wall_top = (SCREEN_HEIGHT as f32 - wall_height) / 2.0;

            canvas.set_draw_color(Color::RGB(100, 100, 100));
            canvas
                .fill_rect(Rect::new(x as i32, wall_top as i32, 1, wall_height as u32))
                .unwrap();
        }
    }

    canvas.set_draw_color(Color::RGB(0, 0, 255));
    for (_id, (pos, _rot)) in other_players.iter() {
        let dx = pos.x - player.x;
        let dy = pos.y - player.y;
        let distance = (dx * dx + dy * dy).sqrt();
        if distance > RAY_DISTANCE {
            continue;
        }

        if !has_line_of_sight(maze, (player.x, player.y), (pos.x, pos.y)) {
            continue;
        }

        let angle_to_enemy = dy.atan2(dx);
        let mut angle_diff = angle_to_enemy - player.angle;
        while angle_diff > std::f32::consts::PI {
            angle_diff -= 2.0 * std::f32::consts::PI;
        }
        while angle_diff < -std::f32::consts::PI {
            angle_diff += 2.0 * std::f32::consts::PI;
        }

        if angle_diff.abs() > FOV / 2.0 {
            continue;
        }

        let screen_x = ((angle_diff + FOV / 2.0) / FOV) * SCREEN_WIDTH as f32;
        let sprite_height = (SCREEN_HEIGHT as f32 / distance).min(SCREEN_HEIGHT as f32 / 1.5);
        let sprite_width = sprite_height / 2.0;
        let top = (SCREEN_HEIGHT as f32 - sprite_height) / 2.0;

        let rect = Rect::new(
            (screen_x - sprite_width / 2.0) as i32,
            top as i32,
            sprite_width as u32,
            sprite_height as u32,
        );

        let _ = canvas.fill_rect(rect);
    }
}

fn render_health_bar(canvas: &mut Canvas<Window>, health: u32) {
    let max_width = 200;
    let height = 20;
    canvas.set_draw_color(Color::RGB(60, 60, 60));
    let _ = canvas.fill_rect(Rect::new(20, 20, max_width, height));

    let health_width = ((health as f32 / 100.0) * max_width as f32).round() as u32;
    canvas.set_draw_color(Color::RGB(200, 30, 30));
    let _ = canvas.fill_rect(Rect::new(20, 20, health_width, height));
}

fn render_minimap_below(
    canvas: &mut Canvas<Window>,
    maze: &[[Tile; MAZE_WIDTH]; MAZE_HEIGHT],
    player: &Player3D,
    other_players: &HashMap<String, (Position, Rotation)>,
) {
    let minimap_width = (MAZE_WIDTH * MINIMAP_TILE_SIZE as usize) as u32;
    let minimap_height = (MAZE_HEIGHT * MINIMAP_TILE_SIZE as usize) as u32;
    let offset_x = ((SCREEN_WIDTH - minimap_width) / 2) as i32;
    let offset_y = (SCREEN_HEIGHT - minimap_height) as i32 - 10;

    for (y, row) in maze.iter().enumerate() {
        for (x, tile) in row.iter().enumerate() {
            let color = match tile {
                Tile::Wall => Color::RGB(80, 80, 80),
                Tile::Floor => Color::RGB(200, 200, 200),
            };

            canvas.set_draw_color(color);
            let _ = canvas.fill_rect(Rect::new(
                offset_x + (x * MINIMAP_TILE_SIZE as usize) as i32,
                offset_y + (y * MINIMAP_TILE_SIZE as usize) as i32,
                MINIMAP_TILE_SIZE,
                MINIMAP_TILE_SIZE,
            ));
        }
    }

    canvas.set_draw_color(Color::RGB(255, 0, 0));
    let _ = canvas.fill_rect(Rect::new(
        offset_x + (player.x as usize * MINIMAP_TILE_SIZE as usize) as i32,
        offset_y + (player.y as usize * MINIMAP_TILE_SIZE as usize) as i32,
        MINIMAP_TILE_SIZE,
        MINIMAP_TILE_SIZE,
    ));

    canvas.set_draw_color(Color::RGB(0, 0, 255));
    for (_name, (pos, _rot)) in other_players.iter() {
        let _ = canvas.fill_rect(Rect::new(
            offset_x + (pos.x as usize * MINIMAP_TILE_SIZE as usize) as i32,
            offset_y + (pos.y as usize * MINIMAP_TILE_SIZE as usize) as i32,
            MINIMAP_TILE_SIZE,
            MINIMAP_TILE_SIZE,
        ));
    }
}

fn find_target_in_crosshair(
    player: &Player3D,
    others: &HashMap<String, (Position, Rotation)>,
) -> Option<(String, Position)> {
    let mut best_target: Option<(String, Position)> = None;
    let mut closest_angle = std::f32::consts::PI;

    for (name, (pos, _rot)) in others.iter() {
        let dx = pos.x - player.x;
        let dy = pos.y - player.y;
        let distance = (dx * dx + dy * dy).sqrt();
        if distance > 30.0 {
            continue;
        }

        let angle_to = dy.atan2(dx);
        let mut angle_diff = angle_to - player.angle;

        while angle_diff > std::f32::consts::PI {
            angle_diff -= 2.0 * std::f32::consts::PI;
        }
        while angle_diff < -std::f32::consts::PI {
            angle_diff += 2.0 * std::f32::consts::PI;
        }

        if angle_diff.abs() < 0.2 && angle_diff.abs() < closest_angle {
            closest_angle = angle_diff.abs();
            best_target = Some((name.clone(), *pos));
        }
    }

    best_target
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    let username = prompt("Enter Name: ");
    let server_addr = prompt("Enter IP Address (example 127.0.0.1:2025): ");

    let client = NetworkClient::new("0.0.0.0:0", &server_addr)?;
    client.send(&ClientMessage::JoinGame {
        username: username.clone(),
    })?;

    let sdl_context = sdl2::init()?;
    let ttf_context = sdl2::ttf::init()?; // FONT INIT
    let video_subsystem = sdl_context.video()?;

    let window = video_subsystem
        .window("Maze Wars", SCREEN_WIDTH, SCREEN_HEIGHT)
        .position_centered()
        .build()?;

    let mut canvas = window.into_canvas().build()?;
    let texture_creator = canvas.texture_creator();

    let mut event_pump = sdl_context.event_pump()?;
    let font = ttf_context.load_font("assets/fonts/FiraSans-Bold.ttf", 64)?;

    let maze_map = generate_maze();
    let mut player = Player3D {
        x: 1.5,
        y: 1.5,
        angle: 0.0,
    };

    let mut running = true;
    let mut game_started = false;
    let mut player_dead = false;
    let mut other_players: HashMap<String, (Position, Rotation)> = HashMap::new();
    let mut player_health = 100;
    let mut last_frame = Instant::now();
    let mut last_sent_position = Position::default();
    let mut last_sent_rotation = Rotation::default();

    while running {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => running = false,

                // Movement
                Event::KeyDown {
                    keycode: Some(Keycode::W),
                    ..
                } if game_started && !player_dead => {
                    let new_x = player.x + player.angle.cos() * 0.1;
                    let new_y = player.y + player.angle.sin() * 0.1;
                    if maze_map[new_y as usize][new_x as usize] != Tile::Wall {
                        player.x = new_x;
                        player.y = new_y;
                    }
                }
                Event::KeyDown {
                    keycode: Some(Keycode::S),
                    ..
                } if game_started && !player_dead => {
                    let new_x = player.x - player.angle.cos() * 0.1;
                    let new_y = player.y - player.angle.sin() * 0.1;
                    if maze_map[new_y as usize][new_x as usize] != Tile::Wall {
                        player.x = new_x;
                        player.y = new_y;
                    }
                }
                Event::KeyDown {
                    keycode: Some(Keycode::A),
                    ..
                } if game_started && !player_dead => {
                    player.angle -= 0.1;
                }
                Event::KeyDown {
                    keycode: Some(Keycode::D),
                    ..
                } if game_started && !player_dead => {
                    player.angle += 0.1;
                }

                // Shooting
                Event::KeyDown {
                    keycode: Some(Keycode::Space),
                    ..
                } if game_started && !player_dead => {
                    if let Some((target, _)) = find_target_in_crosshair(&player, &other_players) {
                        client.send(&ClientMessage::ShotPlayer {
                            player_username: target,
                        })?;
                    }
                }
                _ => {}
            }
        }

        if let Some(msg) = client.try_receive() {
            match msg {
                ServerMessage::GameStart => game_started = true,
                ServerMessage::HealthUpdate { player_id, health } => {
                    if player_id == username {
                        player_health = health;
                    }
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
                ServerMessage::PlayerDeath { player_id, .. } => {
                    if player_id == username {
                        player_dead = true;
                        println!("💀 You were killed!");
                    } else {
                        println!("⚰️ {} was eliminated!", player_id);
                        other_players.remove(&player_id);
                    }
                }
            }
        }

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

        if !player_dead && (position != last_sent_position || rotation != last_sent_rotation) {
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
            render_first_person_view(&mut canvas, &maze_map, &player, &other_players);
            if !player_dead {
                render_health_bar(&mut canvas, player_health);
                render_minimap_below(&mut canvas, &maze_map, &player, &other_players);
            } else {
                let surface = font.render("YOU DIED").blended(Color::RGB(255, 0, 0))?;
                let texture = texture_creator.create_texture_from_surface(&surface)?;
                let rect = Rect::new(250, 250, 300, 100);
                canvas.copy(&texture, None, Some(rect))?;
            }
        }

        canvas.present();

        let elapsed = last_frame.elapsed();
        if elapsed < Duration::from_millis(16) {
            std::thread::sleep(Duration::from_millis(16) - elapsed);
        }
        last_frame = Instant::now();
    }

    Ok(())
}

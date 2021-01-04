#![windows_subsystem = "windows"]
use ggez;
use ggez::event::{self, EventHandler, KeyCode};
use ggez::timer;
use ggez::graphics::{self, Image};
use ggez::{Context, GameResult};
use ggez::conf::WindowMode;
use ggez::input::mouse::{self, MouseButton};
use ggez::input::keyboard;
use ggez::nalgebra as na;
use ggez::mint;

use std::path;
use std::env;

const SCREEN_WIDTH: usize = 640;
const SCREEN_HEIGHT: usize = 480;
const DESIRED_FPS: u32 = 30;

const WORLD_WIDTH: usize = 24;
const WORLD_HEIGHT: usize = 24;

const WORLD: [[usize; WORLD_WIDTH]; WORLD_HEIGHT] = [
    [1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1],
    [1,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1],
    [1,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1],
    [1,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1],
    [1,0,0,0,0,0,2,2,2,2,2,0,0,0,0,3,0,3,0,3,0,0,0,1],
    [1,0,0,0,0,0,2,0,0,0,2,0,0,0,0,0,0,0,0,0,0,0,0,1],
    [1,0,0,0,0,0,2,0,0,0,2,0,0,0,0,3,0,0,0,3,0,0,0,1],
    [1,0,0,0,0,0,2,0,0,0,2,0,0,0,0,0,0,0,0,0,0,0,0,1],
    [1,0,0,0,0,0,2,2,0,2,2,0,0,0,0,3,0,3,0,3,0,0,0,1],
    [1,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1],
    [1,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1],
    [1,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1],
    [1,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1],
    [1,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1],
    [1,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1],
    [1,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1],
    [1,4,4,4,4,4,4,4,4,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1],
    [1,4,0,4,0,0,0,0,4,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1],
    [1,4,0,0,0,0,5,0,4,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1],
    [1,4,0,4,0,0,0,0,4,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1],
    [1,4,0,4,4,4,4,4,4,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1],
    [1,4,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1],
    [1,4,4,4,4,4,4,4,4,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1],
    [1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1]
    ];

const COLOR_RED_BRIGHT: [u8; 4] = [255, 0, 0, 255];
const COLOR_RED_DARK: [u8; 4] = [128, 0, 0, 255];
const COLOR_BLUE_BRIGHT: [u8; 4] = [0, 0, 255, 255];
const COLOR_BLUE_DARK: [u8; 4] = [0, 0, 128, 255];
const COLOR_GREEN_BRIGHT: [u8; 4] = [0, 128, 0, 255];
const COLOR_GREEN_DARK: [u8; 4] = [0, 64, 0, 255];
const COLOR_WHITE_BRIGHT: [u8; 4] = [255, 255, 255, 255];
const COLOR_WHITE_DARK: [u8; 4] = [128, 128, 128, 255];
const COLOR_YELLOW_BRIGHT: [u8; 4] = [255, 255, 0, 255];
const COLOR_YELLOW_DARK: [u8; 4] = [128, 128, 0, 255];

struct MainState {
    pos: na::Point2<f64>,
    dir: na::Vector2<f64>,
    plane: na::Vector2<f64>,
    image: Image,
    mouse_position: Option<mint::Point2<f32>>,
}

impl MainState {
    fn new(ctx: &mut Context) -> GameResult<Self> {
        let buffer = vec![0; 4 * SCREEN_WIDTH * SCREEN_HEIGHT];
        let image = Image::from_rgba8(ctx, SCREEN_WIDTH as u16, SCREEN_HEIGHT as u16, &buffer)?;

        Ok( MainState {
            pos: na::Point2::new(22.0, 12.0),
            dir: na::Vector2::new(-1.0, 0.0),
            plane: na::Vector2::new(0.0, 0.66),
            image,
            mouse_position: None,
        })
    }

    // returns a buffer
    fn raycast(&self) -> Vec<u8> {
        let mut buffer = vec![0; 4 * SCREEN_WIDTH * SCREEN_HEIGHT];

        for x in 0..SCREEN_WIDTH {
            // calculate ray position and direction
            let camera_x = 2.0 * x as f64 / SCREEN_WIDTH as f64 - 1.0; // x-coordinate in camera space
            let ray_dir_x = self.dir.x + self.plane.x * camera_x;
            let ray_dir_y = self.dir.y + self.plane.y * camera_x;
            
            // which box of the world we're in
            let mut map_x = self.pos.x;
            let mut map_y = self.pos.y;

            // length of ray from on x or y-side to next x or y-side
            let delta_dist_x = if !ray_dir_y.is_normal() { 0.0 } else { if !ray_dir_x.is_normal() { 1.0 } else { (1.0 / ray_dir_x).abs() }};
            let delta_dist_y = if !ray_dir_x.is_normal() { 0.0 } else { if !ray_dir_y.is_normal() { 1.0 } else { (1.0 / ray_dir_y).abs() }};

            // step:       what direction to step in x or y-direction (either 1 or -1)
            // side_dist:  length of ray from current position to next x or y-side
            let (step_x, mut side_dist_x) = if ray_dir_x < 0.0 {
                (-1.0, (self.pos.x - map_x) as f64 * delta_dist_x)
            } else {
                (1.0, (map_x + 1.0 - self.pos.x) as f64 * delta_dist_x)
            };
            let (step_y, mut side_dist_y) = if ray_dir_y < 0.0 {
                (-1.0, (self.pos.y - map_y) as f64 * delta_dist_y)
            } else {
                (1.0, (map_y + 1.0 - self.pos.y) as f64 * delta_dist_y)
            };

            let mut hit = false; // was there a wall hit?
            let mut north_south = false; // was a NS or EW wall hit

            // perform DDA algorithm (simple but effective way of sending rays out and detecting hits)
            while !hit {
                // jump to next map square, OR in x-direction, OR in y-direction
                if side_dist_x < side_dist_y {
                    side_dist_x += delta_dist_x;
                    map_x += step_x;
                    north_south = false;
                } else {
                    side_dist_y += delta_dist_y;
                    map_y += step_y;
                    north_south = true;
                }

                // check if ray has hit a wall
                if WORLD[map_y as usize][map_x as usize] > 0 {
                    hit = true;
                }
            }

            // calculate distance projected on camera direction (Euclidean distance will give fisheye effect!)
            let perp_wall_dist = if north_south {
                (map_y - self.pos.y + (1.0 - step_y) / 2.0) / ray_dir_y
            } else {
                (map_x - self.pos.x + (1.0 - step_x) / 2.0) / ray_dir_x
            };

            // calculate height of line to draw on screen
            let line_height = (SCREEN_HEIGHT as f64 / perp_wall_dist) as i64;
            let h = SCREEN_HEIGHT as i64;

            // calculate lowest and highest pixel to fill in current stripe
            let mut draw_start = -line_height / 2 + h / 2;
            if draw_start < 0 {
                draw_start = 0;
            }
            let mut draw_end = line_height / 2 + h / 2;
            if draw_end >= h {
                draw_end = h - 1;
            }

            // choose wall color, darker for y-side
            let color = match WORLD[map_y as usize][map_x as usize] {
                1 => if north_south { COLOR_RED_DARK } else { COLOR_RED_BRIGHT },
                2 => if north_south { COLOR_GREEN_DARK } else { COLOR_GREEN_BRIGHT },
                3 => if north_south { COLOR_BLUE_DARK } else { COLOR_BLUE_BRIGHT },
                4 => if north_south { COLOR_WHITE_DARK } else { COLOR_WHITE_BRIGHT },
                _ => if north_south { COLOR_YELLOW_DARK } else { COLOR_YELLOW_BRIGHT },
            };

            // draw the pixels of the stripe as a vertical line
            for y in (draw_start as usize)..(draw_end as usize) {
                for i in 0..4 {
                    let buffer_index = y * SCREEN_WIDTH * 4 + x * 4 + i;
                    buffer[buffer_index] = color[i];
                }
            }

        }
        buffer
    }

    fn update_image(&mut self, buffer: Vec<u8>, ctx: &mut Context) -> GameResult {
        self.image = Image::from_rgba8(ctx, SCREEN_WIDTH as u16, SCREEN_HEIGHT as u16, &buffer)?;
        Ok(())
    }

    fn mouse_motion_event(&mut self, ctx: &mut Context) {
        let is_pressed = mouse::button_pressed(ctx, MouseButton::Left);
        let current_pos = mouse::position(ctx);
        if is_pressed {
            if let Some(m_pos) = self.mouse_position {
                let diff = mint::Point2 {x: m_pos.x - current_pos.x, y: m_pos.y - current_pos.y};
                if diff.x != 0.0 || diff.y != 0.0 {
                    let window = graphics::window(ctx);
                    let mut window_pos = window.get_position().unwrap();
                    window_pos.x -= diff.x as f64;
                    window_pos.y -= diff.y as f64;
                    window.set_position(window_pos);
                }
            } else {
                self.mouse_position = Some(current_pos)
            } 
        } else {
            self.mouse_position = None;
        }
    }

    fn key_event(&mut self, ctx: &mut Context) {
        const MOVE_SPEED: f64 = 5.0 / DESIRED_FPS as f64;
        const ROTATE_SPEED: f64 = 3.0 / DESIRED_FPS as f64;

        // move forward if no wall in front of player
        if keyboard::is_key_pressed(ctx, KeyCode::W) {
            let dx = self.pos.x + self.dir.x * MOVE_SPEED;
            let dy = self.pos.y + self.dir.y * MOVE_SPEED;
            if WORLD[self.pos.y as usize][dx as usize] == 0 { self.pos.x = dx; }
            if WORLD[dy as usize][self.pos.x as usize] == 0 { self.pos.y = dy; }
        }

        // move backwards if no wall behind
        if keyboard::is_key_pressed(ctx, KeyCode::S) {
            let dx = self.pos.x - self.dir.x * MOVE_SPEED;
            let dy = self.pos.y - self.dir.y * MOVE_SPEED;
            if WORLD[self.pos.y as usize][dx as usize] == 0 { self.pos.x = dx; }
            if WORLD[dy as usize][self.pos.y as usize] == 0 { self.pos.y = dy; }
        }

        // rotate right
        if keyboard::is_key_pressed(ctx, KeyCode::D) {
            // both camera direction and plane must be rotated
            let old_dir_x = self.dir.x;
            self.dir.x = self.dir.x * (-ROTATE_SPEED).cos() - self.dir.y * (-ROTATE_SPEED).sin();
            self.dir.y = old_dir_x * (-ROTATE_SPEED).sin() + self.dir.y * (-ROTATE_SPEED).cos();
            let old_plane_x = self.plane.x;
            self.plane.x = self.plane.x * (-ROTATE_SPEED).cos() - self.plane.y * (-ROTATE_SPEED).sin();
            self.plane.y = old_plane_x * (-ROTATE_SPEED).sin() + self.plane.y * (-ROTATE_SPEED).cos();
        }
    
        // rotate left
        if keyboard::is_key_pressed(ctx, KeyCode::A) {
            // both camera direction and plane must be rotated
            let old_dir_x = self.dir.x;
            self.dir.x = self.dir.x * ROTATE_SPEED.cos() - self.dir.y * ROTATE_SPEED.sin();
            self.dir.y = old_dir_x * ROTATE_SPEED.sin() + self.dir.y * ROTATE_SPEED.cos();
            let old_plane_x = self.plane.x;
            self.plane.x = self.plane.x * ROTATE_SPEED.cos() - self.plane.y * ROTATE_SPEED.sin();
            self.plane.y = old_plane_x * ROTATE_SPEED.sin() + self.plane.y * ROTATE_SPEED.cos();
        }
    }
}

impl EventHandler for MainState {
    fn update(&mut self, ctx: &mut Context) -> GameResult {
        while timer::check_update_time(ctx, DESIRED_FPS) {
            self.mouse_motion_event(ctx);
            self.key_event(ctx);
            let buffer = self.raycast();
            self.update_image(buffer, ctx)?;
        }
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        graphics::clear(ctx, [0.0, 0.0, 0.0, 1.0].into());
        graphics::draw(
            ctx,
            &self.image,
            (na::Point2::new(0.0, 0.0,),))?;

        graphics::present(ctx)?;
        Ok(())
    }
}

fn main() -> GameResult {
    println!("Hello game!");
    let mut cb = ggez::ContextBuilder::new("shoota", "shoota")
        .window_mode( WindowMode {
            width: SCREEN_WIDTH as f32,
            height: SCREEN_HEIGHT as f32,
            resizable: false,
            borderless: true,
            ..WindowMode::default()
        });
    if let Ok(manifest_dir) = env::var("CARGO_MANIFEST_DIR") {
        let mut path = path::PathBuf::from(manifest_dir);
        path.push("resources");
        cb = cb.add_resource_path(path);
    }

    let (ctx, event_loop) = &mut cb.build()?;
    graphics::set_window_title(ctx, "shoota - final showdown with cow & pig");
    mouse::set_cursor_hidden(ctx, true);

    let state = &mut MainState::new(ctx)?;
    event::run(ctx, event_loop, state)
}
use ggez;
use ggez::event::{self, EventHandler, KeyCode};
use ggez::graphics::{self, Image};
use ggez::{Context, GameResult, GameError, timer};
use ggez::conf::WindowMode;
use ggez::input::{mouse, keyboard};

use glam::*;

use std::path;
use std::env;

const SCREEN_WIDTH: usize = 1024; //640;
const SCREEN_HEIGHT: usize = 768; //480;
const MAP_WIDTH: usize = 24;
const MAP_HEIGHT: usize = 24;
const DESIRED_FPS: u32 = 30;
const TEX_WIDTH: usize = 64;
const TEX_HEIGHT: usize = 64;

const WORLD: [[i32; MAP_WIDTH]; MAP_HEIGHT] =
[
  [4,4,4,4,4,4,4,4,4,4,4,4,4,4,4,4,7,7,7,7,7,7,7,7],
  [4,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,7,0,0,0,0,0,0,7],
  [4,0,1,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,7],
  [4,0,2,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,7],
  [4,0,3,0,0,0,0,0,0,0,0,0,0,0,0,0,7,0,0,0,0,0,0,7],
  [4,0,4,0,0,0,0,5,5,5,5,5,5,5,5,5,7,7,0,7,7,7,7,7],
  [4,0,5,0,0,0,0,5,0,5,0,5,0,5,0,5,7,0,0,0,7,7,7,1],
  [4,0,6,0,0,0,0,5,0,0,0,0,0,0,0,5,7,0,0,0,0,0,0,8],
  [4,0,7,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,7,7,7,1],
  [4,0,8,0,0,0,0,5,0,0,0,0,0,0,0,5,7,0,0,0,0,0,0,8],
  [4,0,0,0,0,0,0,5,0,0,0,0,0,0,0,5,7,0,0,0,7,7,7,1],
  [4,0,0,0,0,0,0,5,5,5,5,0,5,5,5,5,7,7,7,7,7,7,7,1],
  [6,6,6,6,6,6,6,6,6,6,6,0,6,6,6,6,6,6,6,6,6,6,6,6],
  [8,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,4],
  [6,6,6,6,6,6,0,6,6,6,6,0,6,6,6,6,6,6,6,6,6,6,6,6],
  [4,4,4,4,4,4,0,4,4,4,6,0,6,2,2,2,2,2,2,2,3,3,3,3],
  [4,0,0,0,0,0,0,0,0,4,6,0,6,2,0,0,0,0,0,2,0,0,0,2],
  [4,0,0,0,0,0,0,0,0,0,0,0,6,2,0,0,5,0,0,2,0,0,0,2],
  [4,0,0,0,0,0,0,0,0,4,6,0,6,2,0,0,0,0,0,2,2,0,2,2],
  [4,0,6,0,6,0,0,0,0,4,6,0,0,0,0,0,5,0,0,0,0,0,0,2],
  [4,0,0,5,0,0,0,0,0,4,6,0,6,2,0,0,0,0,0,2,2,0,2,2],
  [4,0,6,0,6,0,0,0,0,4,6,0,6,2,0,0,5,0,0,2,0,0,0,2],
  [4,0,0,0,0,0,0,0,0,4,6,0,6,2,0,0,0,0,0,2,0,0,0,2],
  [4,4,4,4,4,4,4,4,4,4,1,1,1,2,2,2,2,2,2,3,3,3,3,3]
];

struct MainState {
  pos: DVec2,
  dir: DVec2,
  plane: DVec2,
  image: Image,
  textures: Vec<Vec<Vec<u8>>>,
}

impl MainState {
  fn new(ctx: &mut Context) -> GameResult<Self> {
    let buffer = vec![0; 4 * SCREEN_WIDTH * SCREEN_HEIGHT];
    let image = Image::from_rgba8(ctx, SCREEN_WIDTH as u16, SCREEN_HEIGHT as u16, &buffer)?;
    let textures = load_all_textures(ctx)?;

    Ok(Self {
      pos: DVec2::new(12.0, 12.0), // x and y start position
      dir: DVec2::new(-1.0, 0.0),  // initial direction vector
      plane: DVec2::new(0.0, 0.66), // the 2d raycaster version of camera plane
      image,
      textures
    })
  }

  fn raycast_to_buffer(&self) -> Vec<u8> {
    let mut buffer = vec![0; 4 * SCREEN_WIDTH * SCREEN_HEIGHT];

    for x in 0..SCREEN_WIDTH {
      // calculate ray position and direction
      let camera_x = (2 * x) as f64 / SCREEN_WIDTH as f64 - 1.0;
      let ray_dir = DVec2::new(
        self.dir.x + self.plane.x * camera_x,
        self.dir.y + self.plane.y * camera_x
      );
      
      let delta_dist_x = if !ray_dir.x.is_normal() { 1.0e10_f64 } else { (1.0 / ray_dir.x).abs() };
      let delta_dist_y = if !ray_dir.y.is_normal() { 1.0e10_f64 } else { (1.0 / ray_dir.y).abs() };

      let mut map_x = self.pos.x as i32;
      let mut map_y = self.pos.y as i32;
      let mut side_hit;
      
      let (step_x, mut side_dist_x) = 
        if ray_dir.x < 0.0 {
          (-1, (self.pos.x - map_x as f64) * delta_dist_x)
        } else {
          (1, (map_x as f64 + 1.0 - self.pos.x) * delta_dist_x)
        };
      let (step_y, mut side_dist_y) =
        if ray_dir.y < 0.0 {
          (-1, (self.pos.y - map_y as f64) * delta_dist_y)
        } else {
          (1, (map_y as f64 + 1.0 - self.pos.y) * delta_dist_y)
        };
      
      // les go, DDA
      loop {
        // jump to next map square, either in x or y direction
        if side_dist_x < side_dist_y {
          side_dist_x += delta_dist_x;
          map_x += step_x;
          side_hit = false;
        } else {
          side_dist_y += delta_dist_y;
          map_y += step_y;
          side_hit = true;
        }

        // check if ray has hit a wall
        if WORLD[map_y as usize][map_x as usize] > 0 {
          break;
        }
      }
      
      let perp_wall_dist = if side_hit {
        side_dist_y - delta_dist_y
        //(map_y as f64 - self.pos.y + (1.0 - step_y as f64) / 2.0) / ray_dir.y
      } else {
        side_dist_x - delta_dist_x
        //(map_x as f64 - self.pos.x + (1.0 - step_x as f64) / 2.0) / ray_dir.x
      };
      //println!("side_hit (use y): {}, perp_wall_dist: {}, ray_dir.y: {}, ray_dir.x: {}", side_hit, perp_wall_dist, ray_dir.y, ray_dir.x);
      //println!("mapy: {}, posy: {}, step_y: {}, map_y - posy + (1 - stepy) = {}", map_y as f64, self.pos.y, step_y as f64, (map_y as f64 - self.pos.y + (1.0 - step_y as f64)));
      //println!("({} - {} + (1 - {}) / 2) / {} = {}", map_y as f64, self.pos.y, step_y as f64, ray_dir.y, perp_wall_dist);
      //println!("mapx: {}, posx: {}, step_x: {}, mapx - posx + (1 - stepx) = {}", map_x as f64, self.pos.x, step_x as f64, (map_x as f64 - self.pos.x + (1.0 - step_x as f64)));

      //println!("side_hit: {}", side_hit);
      //println!("mapy: {}, pos.y: {}, step_y: {}, ray_dir.y: {}", map_y, self.pos.y, step_y, ray_dir.y);
      //println!("mapx: {}, pos.x: {}, step_x: {}, ray_dir.x: {}", map_x, self.pos.x, step_x, ray_dir.x);
      //println!("mapy - pos.y + (1.0 - stepy) / 2.0 = {}", (map_y as f64 - self.pos.y + (1.0 - step_y as f64) / 2.0));  
      //println!("mapx - pos.x + (1.0 - stepx) / 2.0 = {}", (map_x as f64 - self.pos.x + (1.0 - step_x as f64) / 2.0));  
    
      // calculate height of line to draw on screen
      let line_height = (SCREEN_HEIGHT as f64 / perp_wall_dist) as i32;
      //println!("screen_h: {}, perp_wall_dist: {}, h / p: {}", SCREEN_HEIGHT, perp_wall_dist, SCREEN_HEIGHT as f64 / perp_wall_dist);
      //println!("line_height: {}, perp_wall_dist: {}", line_height, perp_wall_dist);
      // calclulate lowest and highest pixel to fill in current stripe
      let h = SCREEN_HEIGHT as i32;
      let draw_start = (-line_height / 2 + h / 2).max(0);
      let draw_end = (line_height / 2 + h / 2).min(h - 1);
      
      // texturing calculations
      let tex_id = (WORLD[map_y as usize][map_x as usize] - 1) as usize;

      // calculate value of wall_x
      // where exactly the wall was hit
      let mut wall_x: f64 = if side_hit {
        self.pos.x + perp_wall_dist * ray_dir.x
      } else {
        self.pos.y + perp_wall_dist * ray_dir.y
      };
      wall_x -= wall_x.floor();

      // x coord on the texture
      let tex_x = if (!side_hit && ray_dir.x > 0.0) || (side_hit && ray_dir.y < 0.0) {
        TEX_WIDTH - 1 - (wall_x * TEX_WIDTH as f64) as usize
      } else {
        (wall_x * TEX_WIDTH as f64) as usize
      };

      // how much to increase the texture coordinate per screen pixel
      let step = TEX_HEIGHT as f64 / line_height as f64;
      // starting texture coordinate
      let mut tex_pos = (draw_start - h / 2 + line_height / 2) as f64 * step;

      //println!("draw_start: {}", draw_start);
      //println!("draw_end:   {}", draw_end);
      // put stuff in buffer
      for y in draw_start as usize..draw_end as usize {
        let tex_y = tex_pos as usize & (TEX_HEIGHT - 1);
        tex_pos += step;
        let buffer_index = y * SCREEN_WIDTH * 4 + x * 4;
        let tex_index = TEX_HEIGHT * tex_y + tex_x;
        if buffer_index < buffer.len() - 4 {
          buffer[buffer_index]     = self.textures[tex_id][tex_index][0] >> side_hit as u8;
          buffer[buffer_index + 1] = self.textures[tex_id][tex_index][1] >> side_hit as u8;
          buffer[buffer_index + 2] = self.textures[tex_id][tex_index][2] >> side_hit as u8;
          buffer[buffer_index + 3] = self.textures[tex_id][tex_index][3] >> side_hit as u8;
        }
      }
    }
    buffer
  }

  fn update_image(&mut self, buffer: Vec<u8>, ctx: &mut Context) -> GameResult {
    self.image = Image::from_rgba8(ctx, SCREEN_WIDTH as u16, SCREEN_HEIGHT as u16, &buffer)?;
    Ok(())
  }

  fn key_events(&mut self, ctx: &mut Context) {
    const MOVE_SPEED: f64 = 5.0 / DESIRED_FPS as f64;
    const ROTATE_SPEED: f64 = 3.0 / DESIRED_FPS as f64;

    // move forward, W
    if keyboard::is_key_pressed(ctx, KeyCode::W) {
      let dx = self.pos.x + self.dir.x * MOVE_SPEED;
      let dy = self.pos.y + self.dir.y * MOVE_SPEED;
      if WORLD[self.pos.y as usize][dx as usize] == 0 { self.pos.x = dx; }
      if WORLD[dy as usize][self.pos.x as usize] == 0 { self.pos.y = dy; }
    }
    // move backwards, S
    if keyboard::is_key_pressed(ctx, KeyCode::S) {
      let dx = self.pos.x - self.dir.x * MOVE_SPEED;
      let dy = self.pos.y - self.dir.y * MOVE_SPEED;
      if WORLD[self.pos.y as usize][dx as usize] == 0 { self.pos.x = dx; }
      if WORLD[dy as usize][self.pos.x as usize] == 0 { self.pos.y = dy; }
    }
    // rotate right, D
    if keyboard::is_key_pressed(ctx, KeyCode::D) {
      let odx = self.dir.x;
      self.dir.x = odx * (-ROTATE_SPEED).cos() - self.dir.y * (-ROTATE_SPEED).sin();
      self.dir.y = odx * (-ROTATE_SPEED).sin() + self.dir.y * (-ROTATE_SPEED).cos();
      let opx = self.plane.x;
      self.plane.x = opx * (-ROTATE_SPEED).cos() - self.plane.y * (-ROTATE_SPEED).sin();
      self.plane.y = opx * (-ROTATE_SPEED).sin() + self.plane.y * (-ROTATE_SPEED).cos();
    }
    // rotate left, A
    if keyboard::is_key_pressed(ctx, KeyCode::A) {
      let odx = self.dir.x;
      self.dir.x = odx * ROTATE_SPEED.cos() - self.dir.y * ROTATE_SPEED.sin();
      self.dir.y = odx * ROTATE_SPEED.sin() + self.dir.y * ROTATE_SPEED.cos();
      let odx = self.plane.x;
      self.plane.x = odx * ROTATE_SPEED.cos() - self.plane.y * ROTATE_SPEED.sin();
      self.plane.y = odx * ROTATE_SPEED.sin() + self.plane.y * ROTATE_SPEED.cos();
    }
    // strafe left, Q
    if keyboard::is_key_pressed(ctx, KeyCode::Q) {
        let dx = self.pos.x - self.plane.x * MOVE_SPEED;
        let dy = self.pos.y - self.plane.y * MOVE_SPEED;
        if WORLD[self.pos.y as usize][dx as usize] == 0 { self.pos.x = dx; }
        if WORLD[dy as usize][self.pos.x as usize] == 0 { self.pos.y = dy; }
    }
    // strafe right, E
    if keyboard::is_key_pressed(ctx, KeyCode::E) {
        let dx = self.pos.x + self.plane.x * MOVE_SPEED;
        let dy = self.pos.y + self.plane.y * MOVE_SPEED;
        if WORLD[self.pos.y as usize][dx as usize] == 0 { self.pos.x = dx; }
        if WORLD[dy as usize][self.pos.x as usize] == 0 { self.pos.y = dy; }
    }
  }
}

impl EventHandler<GameError> for MainState {
  fn update(&mut self, ctx: &mut Context) -> GameResult {
    while timer::check_update_time(ctx, DESIRED_FPS) {
      self.key_events(ctx);
      let buffer = self.raycast_to_buffer();
      self.update_image(buffer, ctx)?;
    }

    Ok(())
  }

  fn draw(&mut self, ctx: &mut Context) -> GameResult {
    graphics::clear(ctx, [0.0, 0.0, 0.0, 1.0].into());
    graphics::draw(
      ctx,
      &self.image,
      (Vec2::new(0.0, 0.0), )
    )?;

    graphics::present(ctx)?;

    Ok(())
  }
}

fn generate_textures(width: usize, height: usize) -> Vec<Vec<Vec<u8>>> {
  let mut textures: Vec<Vec<Vec<u8>>> = vec![vec![vec![]; width * height]; 8];
  for y in 0..width {
    for x in 0..height {
      let xor_color = (((x * 256) / width) ^ ((y * 256) / height)) as u8;
      let y_color = ((y * 256) / height) as u8;
      let xy_color = ((y * 128) / height + (x * 128) / width) as u8;
      // flat red texture with black cross
      textures[0][width * y + x] = vec![254 * (x != y && x != width - y) as u8, 0, 0, 255];
      // sloped greyscale
      textures[1][width * y + x] = vec![xy_color, xy_color, xy_color, 255];
      // sloped yellow gradient
      textures[2][width * y + x] = vec![xy_color, xy_color, 0, 255];
      // xor greyscale
      textures[3][width * y + x] = vec![xor_color, xor_color, xor_color, 255];
      // xor green
      textures[4][width * y + x] = vec![0, xor_color, 0, 255];
      // red bricks
      textures[5][width * y + x] = vec![192 * (x % 16 > 0 && y % 16 > 0) as u8, 0, 0, 255];
      // red gradient
      textures[6][width * y + x] = vec![y_color, 0, 0, 255];
      // flat grey texture
      textures[7][width * y + x] = vec![128, 128, 128, 255];
    }
  }
  textures
}

fn load_all_textures(ctx: &mut Context) -> GameResult<Vec<Vec<Vec<u8>>>> {
    let mut textures: Vec<Vec<Vec<u8>>> = vec![vec![vec![]; TEX_WIDTH * TEX_HEIGHT]; 8];
    textures[0] = load_image_as_vec("/textures/eagle.png", ctx)?;
    textures[1] = load_image_as_vec("/textures/redbrick.png", ctx)?;
    textures[2] = load_image_as_vec("/textures/purplestone.png", ctx)?;
    textures[3] = load_image_as_vec("/textures/greystone.png", ctx)?;
    textures[4] = load_image_as_vec("/textures/bluestone.png", ctx)?;
    textures[5] = load_image_as_vec("/textures/mossy.png", ctx)?;
    textures[6] = load_image_as_vec("/textures/wood.png", ctx)?;
    textures[7] = load_image_as_vec("/textures/colorstone.png", ctx)?;
    Ok(textures)
}

fn load_image_as_vec(path : &str, ctx: &mut Context) -> GameResult<Vec<Vec<u8>>> {
    let mut tex = vec![vec![]; TEX_WIDTH * TEX_HEIGHT];
    let image = Image::new(ctx, path)?.to_rgba8(ctx)?;
    let mut slice_start = 0;
    for y in 0..TEX_HEIGHT {
        for x in 0..TEX_WIDTH {
            tex[y * TEX_HEIGHT + x] = (&image[slice_start..slice_start + 4])
                .iter()
                .map(|x| x.clone())
                .collect();
            slice_start += 4;
        }
    }
    Ok(tex)
}

fn main() -> GameResult {
  println!("Hallo GAME!!");
  let mut cb = ggez::ContextBuilder::new("shoota2", "shoota2")
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

  let (mut ctx, event_loop) = cb.build()?;
  graphics::set_window_title(&ctx, "shoota2 - the final showdown");
  mouse::set_cursor_hidden(&mut ctx, true);
  
  let state = MainState::new(&mut ctx)?;
  event::run(ctx, event_loop, state)
}

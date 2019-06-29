mod actor;
mod assets;
mod util;

use actor::*;
use assets::Assets;
use util::*;

use ggez::{
    audio::{self, SoundSource},
    conf,
    event::{self, EventHandler, KeyCode, KeyMods},
    graphics, nalgebra as na,
    nalgebra::{Point2, Vector2},
    timer, {Context, ContextBuilder, GameResult},
};

use std::env;
use std::path;

pub const PLAYER_LIFE: f32 = 1.0;
pub const FALL_SPEED: f32 = 7.0;
pub const FLAP_SPEED: f32 = 180.0;
pub const FLAP_TIMEOUT: f32 = 0.25;

pub const DESIRED_FPS: u32 = 60;

pub const PLAYER_BBOX: f32 = 12.0;
pub const PIPE_BBOX: f32 = 12.0;

pub const SCREEN_HEIGHT: f32 = 624.0;
pub const SCREEN_WIDTH: f32 = 1008.0;

#[derive(Debug)]
struct InputState {
    flap: bool,
}

impl Default for InputState {
    fn default() -> Self {
        InputState { flap: false }
    }
}

struct MainState {
    player: Actor,
    paused: bool,
    level: i32,
    score: i32,
    assets: Assets,
    screen_width: f32,
    screen_height: f32,
    input: InputState,
    flap_timeout: f32,
}

impl MainState {
    fn new(ctx: &mut Context) -> GameResult<MainState> {
        println!("Game resource path: {:?}", ctx.filesystem);

        print_instructions();

        let assets = Assets::new(ctx)?;
        let player = Actor::new(ActorType::Player);

        let s = MainState {
            player,
            level: 0,
            score: 0,
            assets,
            screen_width: ctx.conf.window_mode.width,
            screen_height: ctx.conf.window_mode.height,
            input: InputState::default(),
            flap_timeout: 0.0,
            paused: true,
        };

        Ok(s)
    }

    fn player_flap(&mut self, dt: f32) {
        self.flap_timeout = crate::FLAP_TIMEOUT;
        let dir = vec_from_angle(0.0);
        let flap_vec = dir * crate::FLAP_SPEED;
        // set constant velocity on flap
        self.player.velocity = flap_vec * dt;
        // makes for more "real" physics but is not flappy bird:
        // player.velocity += flap_vec * dt;
    }

    fn update_player_pos(&mut self, dt: f32) {
        let dir = vec_from_angle(0.0);
        let grav = dir * crate::FALL_SPEED;
        self.player.velocity -= grav * dt;
        // let dv = self.player.velocity * dt;
        self.player.pos += self.player.velocity;
    }


    fn draw_bg(&mut self, ctx: &mut Context) -> GameResult<()> {
        // draw bg
        let bg = &self.assets.bg;
        for tile in 0..=(self.screen_width as u16 / bg.width()) {
            let bg_params =
                graphics::DrawParam::new().dest(Point2::new(f32::from(tile * bg.width()), 0.0));
            graphics::draw(ctx, bg, bg_params)?;
        }
        // draw base
        let base = &self.assets.base;
        for tile in 0..=(self.screen_width as u16 / base.width()) {
            let base_params = graphics::DrawParam::new().dest(Point2::new(
                f32::from(tile * bg.width()),
                f32::from(bg.height()),
            ));
            graphics::draw(ctx, base, base_params)?;
        }

        Ok(())
    }

    fn draw_hud(&mut self, ctx: &mut Context) -> GameResult<()> {
        let score_dest = Point2::new(10.0, 10.0);
        let level_dest = Point2::new(100.0, 10.0);

        let level_str = format!("Level: {}", self.level);
        let score_str = format!("Score: {}", self.score);
        let level_display = graphics::Text::new((level_str, self.assets.font, 20.0));
        let score_display = graphics::Text::new((score_str, self.assets.font, 20.0));
        graphics::draw(ctx, &level_display, (level_dest, 0.0, graphics::WHITE))?;
        graphics::draw(ctx, &score_display, (score_dest, 0.0, graphics::WHITE))?;

        Ok(())
    }

    fn draw_menu(&mut self, ctx: &mut Context) -> GameResult<()> {
        let msg = &self.assets.message;
        let params = graphics::DrawParam::new()
            .dest(translate_coords(
                Point2::origin(),
                self.screen_width,
                self.screen_height - (f32::from(msg.height()) / 2.0) + 35.0,
            ))
            .offset(Point2::new(0.5, 0.5));
        graphics::draw(ctx, msg, params)?;
        Ok(())
    }
    // fn clear_dead_stuff(&mut self) {
    //     self.pipes.retain(|s| s.pos.x > 0.0);
    // }

    // fn update_ui(&mut self, ctx: &mut Context) {
    //     let score_str = format!("Score: {}", self.score);
    //     let level_str = format!("Level: {}", self.level);
    //     let score_text = graphics::Text::new(ctx, &score_str, &self.assets.font).unwrap();
    //     let level_text = graphics::Text::new(ctx, &level_str, &self.assets.font).unwrap();

    //     self.score_display = score_text;
    //     self.level_display = level_text;
    // }
}

fn print_instructions() {
    println!("{:-^60}", "Welcome to Flappy Bird!");
    println!();
    println!("How to play:");
    println!("<space> to flap -- avoid the pipes!");
    println!();
}

/// Translates the world coordinate system, which
/// has Y pointing up and the origin at the center,
/// to the screen coordinate system, which has Y
fn translate_coords(point: Point2<f32>, screen_width: f32, screen_height: f32) -> Point2<f32> {
    let x = point.x + screen_width / 2.0;
    let y = screen_height - (point.y + screen_height / 2.0);
    Point2::new(x, y)
}

fn draw_actor(
    assets: &mut Assets,
    ctx: &mut Context,
    actor: &Actor,
    screen_dims: (f32, f32),
) -> GameResult {
    let pos = translate_coords(actor.pos, screen_dims.0, screen_dims.1);
    let image = assets.actor_image(actor);
    let drawparams = graphics::DrawParam::new()
        .dest(pos)
        .offset(Point2::new(0.5, 0.5));
    graphics::draw(ctx, image, drawparams)
}

fn draw_bird(
    assets: &mut Assets,
    ctx: &mut Context,
    actor: &Actor,
    screen_dims: (f32, f32),
) -> GameResult {
    let pos = translate_coords(actor.pos, screen_dims.0, screen_dims.1);
    let image = assets.actor_image(actor);
    let drawparams = graphics::DrawParam::new()
        .dest(pos)
        .offset(Point2::new(0.5, 0.5));
    graphics::draw(ctx, image, drawparams)
}


impl EventHandler for MainState {
    fn update(&mut self, ctx: &mut Context) -> GameResult {
        while timer::check_update_time(ctx, DESIRED_FPS) {
            if !self.paused {
                let seconds = 1.0 / (DESIRED_FPS as f32);
                self.flap_timeout -= seconds;
                if self.input.flap && self.flap_timeout < 0.0 {
                    self.player_flap(seconds);
                }
                self.update_player_pos(seconds);
            }
        }

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        graphics::clear(ctx, graphics::WHITE);
        self.draw_bg(ctx)?;

        {
            let assets = &mut self.assets;
            let p = &self.player;
            let screen_dims = (self.screen_width, self.screen_height);
            draw_bird(assets, ctx, p, screen_dims)?;
        }

        if self.paused {
            self.draw_menu(ctx)?;
        }

        self.draw_hud(ctx)?;

        graphics::present(ctx)?;
        // And yield the timeslice
        // This tells the OS that we're done using the CPU but it should
        // get back to this program as soon as it can.
        // This ideally prevents the game from using 100% CPU all the time
        // even if vsync is off.
        // The actual behavior can be a little platform-specific.
        timer::yield_now();
        Ok(())
    }

    fn key_down_event(
        &mut self,
        ctx: &mut Context,
        keycode: KeyCode,
        _keymod: KeyMods,
        _repeat: bool,
    ) {
        match keycode {
            KeyCode::A => {
                if self.paused {
                    self.paused = false;
                }
                self.input.flap = true;
            }
            KeyCode::P => {
                let img = graphics::screenshot(ctx).expect("Could not take screenshot");
                img.encode(ctx, graphics::ImageFormat::Png, "/screenshot.png")
                    .expect("Could not save screenshot");
            }
            KeyCode::Escape => ggez::quit(ctx),
            _ => (),
        }
    }

    fn key_up_event(&mut self, _ctx: &mut Context, keycode: KeyCode, _keymod: KeyMods) {
        match keycode {
            KeyCode::A => {
                self.input.flap = false;
            }
            _ => (),
        }
    }
}

pub fn main() -> GameResult {
    let resource_dir = if let Ok(manifest_dir) = env::var("CARGO_MANIFEST_DIR") {
        let mut path = path::PathBuf::from(manifest_dir);
        path.push("resources");
        path
    } else {
        path::PathBuf::from("./resources")
    };

    let cb = ContextBuilder::new("flappy_bird", "cameron.evan@gmail.com")
        .window_setup(conf::WindowSetup::default().title("Flappy Bird!"))
        .window_mode(conf::WindowMode::default().dimensions(SCREEN_WIDTH, SCREEN_HEIGHT))
        .add_resource_path(resource_dir);

    let (ctx, events_loop) = &mut cb.build()?;

    let game = &mut MainState::new(ctx)?;
    event::run(ctx, events_loop, game)
}

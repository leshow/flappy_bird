mod actor;
mod assets;

use actor::*;
use assets::Assets;

use ggez::{
    audio,
    audio::SoundSource,
    conf,
    event::{self, EventHandler, KeyCode, KeyMods},
    graphics, nalgebra as na,
    nalgebra::{Point2, Vector2},
    timer, {Context, ContextBuilder, GameResult},
};

use std::env;
use std::path;

pub const PLAYER_LIFE: f32 = 1.0;
pub const FALL_SPEED: f32 = 2.0;
pub const FLAP_SPEED: f32 = 500.0;
pub const FLAP_HEIGHT: f32 = 5.0;

pub const PLAYER_BBOX: f32 = 12.0;
pub const PIPE_BBOX: f32 = 12.0;

pub const SCREEN_HEIGHT: f32 = 624.0;
pub const SCREEN_WIDTH: f32 = 1008.0;

#[derive(Debug)]
struct InputState {
    xaxis: f32,
    yaxis: f32,
    flap: bool,
}

impl Default for InputState {
    fn default() -> Self {
        InputState {
            xaxis: 0.0,
            yaxis: 0.0,
            flap: false,
        }
    }
}

struct MainState {
    player: Actor,
    level: i32,
    score: i32,
    assets: Assets,
    screen_width: f32,
    screen_height: f32,
    input: InputState,
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
        };

        Ok(s)
    }

    fn draw_bg(&mut self, ctx: &mut Context) -> GameResult<()> {
        let bg = &self.assets.bg;
        for tile in 0..=(SCREEN_WIDTH as u16 / bg.width()) {
            let bg_params =
                graphics::DrawParam::new().dest(Point2::new(f32::from(tile * bg.width()), 0.0));
            graphics::draw(ctx, bg, bg_params)?;
        }
        // draw base
        let base = &self.assets.base;
        for tile in 0..=(SCREEN_WIDTH as u16 / base.width()) {
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
    println!("{:-^30}", "Welcome to Flappy Bird!");
    println!();
    println!("How to play:");
    println!("<space> to flap-- avoid the pipes!");
    println!();
}

/// Translates the world coordinate system, which
/// has Y pointing up and the origin at the center,
/// to the screen coordinate system, which has Y
fn translate_coords(point: Point2<f32>) -> Point2<f32> {
    let x = point.x + SCREEN_WIDTH / 2.0;
    let y = SCREEN_HEIGHT - (point.y + SCREEN_HEIGHT / 2.0);
    Point2::new(x, y)
}

fn draw_actor(assets: &mut Assets, ctx: &mut Context, actor: &Actor) -> GameResult {
    let pos = translate_coords(actor.pos);
    let image = assets.actor_image(actor);
    let drawparams = graphics::DrawParam::new()
        .dest(pos)
        .offset(Point2::new(0.5, 0.5));
    graphics::draw(ctx, image, drawparams)
}

impl EventHandler for MainState {
    fn update(&mut self, ctx: &mut Context) -> GameResult {
        const DESIRED_FPS: u32 = 60;

        while timer::check_update_time(ctx, DESIRED_FPS) {
            let seconds = 1.0 / (DESIRED_FPS as f32);
            if self.input.flap {
                player_flap(&mut self.player, seconds);
            }

            // player_gravity(&mut self.player, seconds);
            update_player_pos(&mut self.player, seconds);
        }

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        graphics::clear(ctx, graphics::WHITE);
        self.draw_bg(ctx)?;

        {
            let assets = &mut self.assets;
            let p = &self.player;
            draw_actor(assets, ctx, p)?;
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

    // Handle key events.  These just map keyboard events
    // and alter our input state appropriately.
    fn key_down_event(
        &mut self,
        ctx: &mut Context,
        keycode: KeyCode,
        _keymod: KeyMods,
        _repeat: bool,
    ) {
        match keycode {
            KeyCode::Space => {
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
            KeyCode::Space => {
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

#[allow(dead_code)]
mod actors;
mod assets;
mod util;

use crate::{
    actors::{Actor, Pipe, Player},
    assets::Assets,
};

use ggez::{
    conf,
    event::{self, EventHandler, KeyCode, KeyMods},
    graphics::{self, DrawParam},
    nalgebra as na,
    nalgebra::Point2,
    timer, {Context, ContextBuilder, GameResult},
};

use std::{env, path};

// game constants
pub const PLAYER_LIFE: f32 = 1.;
pub const FALL_SPEED: f32 = 18.;
pub const FLAP_SPEED: f32 = 320.;
pub const FLAP_TIMEOUT: f32 = 0.35;

pub const DESIRED_FPS: u32 = 60;
pub const MOVE_SPEED: f32 = 2.;

pub const PLAYER_BBOX: f32 = 12.;
pub const PIPE_BBOX: f32 = 12.;

pub const SCREEN_HEIGHT: f32 = 624.;
pub const SCREEN_WIDTH: f32 = 1008.;

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
    player: Player,
    pipes: Vec<(Pipe, Pipe)>, // pipe & upside down pipe
    paused: bool,
    level: i32,
    score: i32,
    assets: Assets,
    screen_width: f32,
    screen_height: f32,
    input: InputState,
    flap_timeout: f32,
    offset: f32,
    frames: u64,
    gameover: bool,
}

impl MainState {
    fn new(ctx: &mut Context) -> GameResult<MainState> {
        println!("Game resource path: {:?}", ctx.filesystem);

        print_instructions();

        let assets = Assets::new(ctx)?;
        let player = Player::new();
        let screen_width = ctx.conf.window_mode.width;
        let screen_height = ctx.conf.window_mode.height;
        let pipes = actors::gen_pipes(&assets, screen_width);

        let s = MainState {
            player,
            pipes,
            level: 0,
            score: 0,
            assets,
            screen_width,
            screen_height,
            input: InputState::default(),
            flap_timeout: 0.,
            paused: true,
            gameover: false,
            offset: 0.,
            frames: 0,
        };

        Ok(s)
    }

    fn restart(&mut self, ctx: &mut Context) -> GameResult<()> {
        *self = MainState::new(ctx)?;
        Ok(())
    }

    fn draw_bg(&mut self, ctx: &mut Context) -> GameResult<()> {
        self.assets.bg.bg.clear();
        let bg = &mut self.assets.bg;
        let first_bg = -1. * (self.offset - (self.offset % f32::from(bg.bg_w)));
        // draw up to 2 panels ahead
        for i in 0..=2 {
            // draw bg
            for tile in 0..=(self.screen_width as u16 / bg.bg_w) {
                let bg_params = DrawParam::new().dest(Point2::new(
                    first_bg + f32::from(i * bg.bg_w) + f32::from(tile * bg.bg_w),
                    0.,
                ));
                bg.bg.add(bg_params);
            }
        }
        graphics::draw(
            ctx,
            &bg.bg,
            DrawParam::new().dest(Point2::new(self.offset, 0.)),
        )?;
        graphics::draw(
            ctx,
            &bg.base,
            DrawParam::new().dest(Point2::new(self.offset, 0.)),
        )?;

        Ok(())
    }

    fn draw_base(&mut self, ctx: &mut Context) -> GameResult<()> {
        self.assets.bg.base.clear();
        let bg = &mut self.assets.bg;
        let first_base = -1. * (self.offset - (self.offset % f32::from(bg.base_w)));
        // draw up to 2 panels ahead
        for i in 0..=2 {
            // draw base
            for tile in 0..=(self.screen_width as u16 / bg.base_w) {
                let base_params = DrawParam::new().dest(Point2::new(
                    first_base + f32::from(i * bg.base_w) + f32::from(tile * bg.base_w),
                    f32::from(bg.bg_h),
                ));
                bg.base.add(base_params);
            }
        }
        graphics::draw(
            ctx,
            &bg.base,
            DrawParam::new().dest(Point2::new(self.offset, 0.)),
        )?;

        Ok(())
    }

    fn draw_pipes(&mut self, ctx: &mut Context) -> GameResult<()> {
        self.assets.bg.pipe.clear();
        let pipe_batch = &mut self.assets.bg.pipe;

        for (btm, top) in &self.pipes {
            // place pipes by the center of their sprite
            let btm_param = DrawParam::new()
                .dest(btm.pos)
                .rotation(btm.facing)
                .offset(Point2::new(0.5, 0.5));

            let top_param = DrawParam::new()
                .dest(top.pos)
                .rotation(top.facing)
                .offset(Point2::new(0.5, 0.5));

            pipe_batch.add(btm_param);
            pipe_batch.add(top_param);
        }

        graphics::draw(
            ctx,
            &self.assets.bg.pipe,
            DrawParam::new().dest(Point2::new(self.offset, 0.)),
        )?;
        Ok(())
    }

    fn draw_hud(&mut self, ctx: &mut Context) -> GameResult<()> {
        let score_dest = Point2::new(10., 10.);
        let level_dest = Point2::new(100., 10.);

        let level_str = format!("Level: {}", self.level);
        let score_str = format!("Score: {}", self.score);
        let level_display = graphics::Text::new((level_str, self.assets.font, 20.));
        let score_display = graphics::Text::new((score_str, self.assets.font, 20.));
        graphics::draw(ctx, &level_display, (level_dest, 0., graphics::WHITE))?;
        graphics::draw(ctx, &score_display, (score_dest, 0., graphics::WHITE))?;

        Ok(())
    }

    fn draw_menu(&mut self, ctx: &mut Context) -> GameResult<()> {
        let msg = &self.assets.message;
        let params = DrawParam::new()
            .dest(translate_coords(
                Point2::origin(),
                self.screen_width,
                self.screen_height - (f32::from(msg.height()) / 2.) + 35.,
            ))
            .offset(Point2::new(0.5, 0.5));
        graphics::draw(ctx, msg, params)?;

        Ok(())
    }

    fn draw_bird(&mut self, ctx: &mut Context) -> GameResult {
        let pos = translate_coords(self.player.pos, self.screen_width, self.screen_height);
        let image = self.assets.player_image(&self.player, self.frames % 15);
        let drawparams = DrawParam::new()
            .dest(pos)
            .rotation(self.player.facing)
            .offset(Point2::new(0.5, 0.5));
        graphics::draw(ctx, image, drawparams)
    }

    fn clear_pipes(&mut self) {
        self.pipes.retain(|s| s.0.pos.x > 0.);
    }

    fn handle_collisions(&mut self) {
        let mut player_pos =
            translate_coords(self.player.pos, self.screen_width, self.screen_height);
        player_pos.x -= self.offset;

        let player_right = player_pos.x + self.player.bbox_size.x;
        let player_top = player_pos.y - self.player.bbox_size.y;
        let player_bottom = player_pos.y + self.player.bbox_size.y;

        let is_hit = |pipe: &Pipe, top: bool| {
            let pipe_right = pipe.pos.x + pipe.bbox_size.x;
            let pipe_left = pipe.pos.x - pipe.bbox_size.x;
            let pipe_top = pipe.pos.y - pipe.bbox_size.y;
            let pipe_bottom = pipe.pos.y + pipe.bbox_size.y;

            let crosses_left = player_right >= pipe_left && player_right <= pipe_right;
            if crosses_left
                && ((top && player_top <= pipe_bottom) || (!top && player_bottom >= pipe_top))
            {
                return true;
            }
            false
        };
        let mut go = false;
        for (btm, top) in &self.pipes {
            if is_hit(top, true) || is_hit(btm, false) {
                go = true;
            }
        }
        self.gameover = go;
    }

    fn draw_game_over(&mut self, ctx: &mut Context) -> GameResult<()> {
        let msg = &self.assets.gameover;
        let params = DrawParam::new()
            .dest(translate_coords(
                Point2::origin(),
                self.screen_width,
                self.screen_height - (f32::from(msg.height()) / 2.),
            ))
            .offset(Point2::new(0.5, 0.5));
        graphics::draw(ctx, msg, params)?;

        Ok(())
    }

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
/// pointing downward and the origin at the top-left,
fn translate_coords(point: Point2<f32>, screen_width: f32, screen_height: f32) -> Point2<f32> {
    let x = point.x + screen_width / 2.;
    let y = screen_height - (point.y + screen_height / 2.);
    Point2::new(x, y)
}

// fn draw_actor(
//     assets: &mut Assets,
//     ctx: &mut Context,
//     actor: &Actor,
//     screen_dims: (f32, f32),
// ) -> GameResult {
//     let pos = translate_coords(actor.pos, screen_dims.0, screen_dims.1);
//     let image = assets.actor_image(actor);
//     let drawparams = DrawParam::new()
//         .dest(pos)
//         .offset(Point2::new(0.5, 0.5));
//     graphics::draw(ctx, image, drawparams)
// }

impl EventHandler for MainState {
    fn update(&mut self, ctx: &mut Context) -> GameResult {
        while timer::check_update_time(ctx, DESIRED_FPS) {
            if !self.paused && !self.gameover {
                let seconds = 1. / (crate::DESIRED_FPS as f32);
                self.flap_timeout -= seconds;
                if self.input.flap && self.flap_timeout < 0. {
                    self.flap_timeout = crate::FLAP_TIMEOUT;
                    self.player.flap(seconds);
                }
                self.offset -= crate::MOVE_SPEED;
                self.frames += 1;
                self.player.update_pos(seconds);
            }
        }

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        graphics::clear(ctx, graphics::WHITE);
        self.draw_bg(ctx)?;
        self.draw_pipes(ctx)?;
        self.draw_base(ctx)?;

        self.draw_bird(ctx)?;

        if self.paused {
            self.draw_menu(ctx)?;
        } else {
            self.handle_collisions();
            if self.gameover {
                self.draw_game_over(ctx)?;
            }
        }

        self.draw_hud(ctx)?;
        self.clear_pipes();

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
            KeyCode::R => {
                if self.gameover {
                    self.restart(ctx).expect("Restart failed");
                }
            }
            KeyCode::Return => {
                self.paused = !self.paused;
            }
            KeyCode::Escape => ggez::quit(ctx),
            _ => (),
        }
    }

    fn key_up_event(&mut self, _ctx: &mut Context, keycode: KeyCode, _keymod: KeyMods) {
        match keycode {
            KeyCode::A => {
                self.input.flap = false;
                // variable height flap
                // let dir = vec_from_angle(0.);
                // let flap_vec = dir * (crate::FLAP_SPEED / 2.0);
                // if self.player.velocity < flap_vec {
                //     self.player.velocity = flap_vec;
                // }
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

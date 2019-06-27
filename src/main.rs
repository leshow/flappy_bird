use ggez::audio;
use ggez::audio::SoundSource;
use ggez::conf;
use ggez::event::{self, EventHandler, KeyCode, KeyMods};
use ggez::graphics;
use ggez::timer;
use ggez::{
    nalgebra as na,
    nalgebra::{Point2, Vector2},
};
use ggez::{Context, ContextBuilder, GameResult};

use std::env;
use std::path;

/// Create a unit vector representing the
/// given angle (in radians)
fn vec_from_angle(angle: f32) -> Vector2<f32> {
    let vx = angle.sin();
    let vy = angle.cos();
    Vector2::new(vx, vy)
}

/// Just makes a random `Vector2` with the given max magnitude.
fn random_vec(max_magnitude: f32) -> Vector2<f32> {
    let angle = rand::random::<f32>() * 2.0 * std::f32::consts::PI;
    let mag = rand::random::<f32>() * max_magnitude;
    vec_from_angle(angle) * (mag)
}

#[derive(Debug, PartialEq, PartialOrd, Ord, Eq)]
enum ActorType {
    Player,
    Pipe,
}

#[derive(Debug, PartialEq, PartialOrd)]
struct Actor {
    ty: ActorType,
    pos: Point2<f32>,
    velocity: Vector2<f32>,
    bbox_size: f32,
}

const PLAYER_LIFE: f32 = 1.0;

const PLAYER_BBOX: f32 = 12.0;
const PIPE_BBOX: f32 = 12.0;

const PIPE_VEL: f32 = 50.0;

impl Actor {
    pub fn new(ty: ActorType) -> Self {
        match ty {
            ActorType::Player => Actor {
                ty,
                pos: Point2::origin(),
                velocity: na::zero(),
                bbox_size: PLAYER_BBOX
            },
            ActorType::Pipe => Actor {
                ty,
                pos: Point2::origin(),
                velocity: na::zero(),
                bbox_size: PIPE_BBOX
            }
        }
    }
}

struct Assets {
    player_image: graphics::Image,
    pipe_image: graphics::Image,
    font: graphics::Font,
//
    shot_sound: audio::SpatialSource,
    hit_sound: audio::SpatialSource,
}

impl Assets {
    fn new(ctx: &mut Context) -> GameResult<Assets> {
        let player_image = graphics::Image::new(ctx, "/redbird-midflap.png")?;
        let pipe_image = graphics::Image::new(ctx, "/pipe-green.png")?;
        let font = graphics::Font::new(ctx, "/DejaVuSerif.ttf")?;

        let mut shot_sound = audio::SpatialSource::new(ctx, "/pew.ogg")?;
        let mut hit_sound = audio::SpatialSource::new(ctx, "/boom.ogg")?;

        shot_sound.set_ears([-1.0, 0.0, 0.0], [1.0, 0.0, 0.0]);
        hit_sound.set_ears([-1.0, 0.0, 0.0], [1.0, 0.0, 0.0]);

        Ok(Assets {
            player_image,
            pipe_image,
            font,
            shot_sound,
            hit_sound,
        })
    }

    fn actor_image(&mut self, actor: &Actor) -> &mut graphics::Image {
        match actor.ty {
            ActorType::Player => &mut self.player_image,
            ActorType::Pipe => &mut self.pipe_image,
        }
    }
}

#[derive(Debug)]
struct InputState {
    xaxis: f32,
    yaxis: f32,
    fire: bool,
}

impl Default for InputState {
    fn default() -> Self {
        InputState {
            xaxis: 0.0,
            yaxis: 0.0,
            fire: false,
        }
    }
}

/// **********************************************************************
/// Now we're getting into the actual game loop.  The `MainState` is our
/// game's "global" state, it keeps track of everything we need for
/// actually running the game.
///
/// Our game objects are simply a vector for each actor type, and we
/// probably mingle gameplay-state (like score) and hardware-state
/// (like `input`) a little more than we should, but for something
/// this small it hardly matters.
/// **********************************************************************

struct MainState {
    player: Actor,
    shots: Vec<Actor>,
    rocks: Vec<Actor>,
    level: i32,
    score: i32,
    assets: Assets,
    screen_width: f32,
    screen_height: f32,
    input: InputState,
    player_shot_timeout: f32,
}

impl MainState {
    fn new(ctx: &mut Context) -> GameResult<MainState> {
        println!("Game resource path: {:?}", ctx.filesystem);

        print_instructions();

        let assets = Assets::new(ctx)?;
        // let score_disp = graphics::Text::new(ctx, "score", &assets.font)?;
        // let level_disp = graphics::Text::new(ctx, "level", &assets.font)?;

        let player = create_player();
        let rocks = create_rocks(5, player.pos, 100.0, 250.0);

        let s = MainState {
            player,
            shots: Vec::new(),
            rocks,
            level: 0,
            score: 0,
            assets,
            screen_width: ctx.conf.window_mode.width,
            screen_height: ctx.conf.window_mode.height,
            input: InputState::default(),
            player_shot_timeout: 0.0,
        };

        Ok(s)
    }

    fn fire_player_shot(&mut self) {
        self.player_shot_timeout = PLAYER_SHOT_TIME;

        let player = &self.player;
        let mut shot = create_shot();
        shot.pos = player.pos;
        let direction = vec_from_angle(shot.facing);
        shot.velocity.x = SHOT_SPEED * direction.x;
        shot.velocity.y = SHOT_SPEED * direction.y;

        self.shots.push(shot);

        let pos = world_to_audio_coords(self.screen_width, self.screen_height, player.pos);
        self.assets.shot_sound.set_position(pos);
        let _ = self.assets.shot_sound.play();
    }

    fn clear_dead_stuff(&mut self) {
        self.shots.retain(|s| s.life > 0.0);
        self.rocks.retain(|r| r.life > 0.0);
    }

    fn handle_collisions(&mut self) {
        for rock in &mut self.rocks {
            let pdistance = rock.pos - self.player.pos;
            if pdistance.norm() < (self.player.bbox_size + rock.bbox_size) {
                self.player.life = 0.0;
            }
            for shot in &mut self.shots {
                let distance = shot.pos - rock.pos;
                if distance.norm() < (shot.bbox_size + rock.bbox_size) {
                    shot.life = 0.0;
                    rock.life = 0.0;
                    self.score += 1;

                    let pos =
                        world_to_audio_coords(self.screen_width, self.screen_height, rock.pos);
                    self.assets.shot_sound.set_position(pos);
                    let _ = self.assets.hit_sound.play();
                }
            }
        }
    }

    fn check_for_level_respawn(&mut self) {
        if self.rocks.is_empty() {
            self.level += 1;
            let r = create_rocks(self.level + 5, self.player.pos, 100.0, 250.0);
            self.rocks.extend(r);
        }
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

/// **********************************************************************
/// A couple of utility functions.
/// **********************************************************************

fn print_instructions() {
    println!("{:-^30}", "Welcome to ASTROBLASTO!");
    println!();
    println!("How to play:");
    println!("L/R arrow keys rotate your ship, up thrusts, space bar fires");
    println!();
}

/// Translates the world coordinate system, which
/// has Y pointing up and the origin at the center,
/// to the screen coordinate system, which has Y
/// pointing downward and the origin at the top-left,
fn world_to_screen_coords(screen_width: f32, screen_height: f32, point: Point2<f32>) -> Point2<f32> {
    let x = point.x + screen_width / 2.0;
    let y = screen_height - (point.y + screen_height / 2.0);
    Point2::new(x, y)
}

fn draw_actor(
    assets: &mut Assets,
    ctx: &mut Context,
    actor: &Actor,
    world_coords: (f32, f32),
) -> GameResult {
    let (screen_w, screen_h) = world_coords;
    let pos = world_to_screen_coords(screen_w, screen_h, actor.pos);
    let image = assets.actor_image(actor);
    let drawparams = graphics::DrawParam::new()
        .dest(pos)
        .offset(Point2::new(0.5, 0.5));
    graphics::draw(ctx, image, drawparams)
}

/// **********************************************************************
/// Now we implement the `EventHandler` trait from `ggez::event`, which provides
/// ggez with callbacks for updating and drawing our game, as well as
/// handling input events.
/// **********************************************************************
impl EventHandler for MainState {
    fn update(&mut self, ctx: &mut Context) -> GameResult {
        const DESIRED_FPS: u32 = 60;

        while timer::check_update_time(ctx, DESIRED_FPS) {
            let seconds = 1.0 / (DESIRED_FPS as f32);

            // Update the player state based on the user input.
            player_handle_input(&mut self.player, &self.input, seconds);
            self.player_shot_timeout -= seconds;
            if self.input.fire && self.player_shot_timeout < 0.0 {
                self.fire_player_shot();
            }

            // Update the physics for all actors.
            // First the player...
            update_actor_position(&mut self.player, seconds);
            wrap_actor_position(
                &mut self.player,
                self.screen_width as f32,
                self.screen_height as f32,
            );

            // Then the shots...
            for act in &mut self.shots {
                update_actor_position(act, seconds);
                wrap_actor_position(act, self.screen_width as f32, self.screen_height as f32);
                handle_timed_life(act, seconds);
            }

            // And finally the rocks.
            for act in &mut self.rocks {
                update_actor_position(act, seconds);
                wrap_actor_position(act, self.screen_width as f32, self.screen_height as f32);
            }

            // Handle the results of things moving:
            // collision detection, object death, and if
            // we have killed all the rocks in the level,
            // spawn more of them.
            self.handle_collisions();

            self.clear_dead_stuff();

            self.check_for_level_respawn();

            // Finally we check for our end state.
            // I want to have a nice death screen eventually,
            // but for now we just quit.
            if self.player.life <= 0.0 {
                println!("Game over!");
                let _ = ggez::quit(ctx);
            }
        }

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        // Our drawing is quite simple.
        // Just clear the screen...
        graphics::clear(ctx, graphics::BLACK);

        // Loop over all objects drawing them...
        {
            let assets = &mut self.assets;
            let coords = (self.screen_width, self.screen_height);

            let p = &self.player;
            draw_actor(assets, ctx, p, coords)?;

            for s in &self.shots {
                draw_actor(assets, ctx, s, coords)?;
            }

            for r in &self.rocks {
                draw_actor(assets, ctx, r, coords)?;
            }
        }

        // And draw the GUI elements in the right places.
        let level_dest = Point2::new(10.0, 10.0);
        let score_dest = Point2::new(200.0, 10.0);

        let level_str = format!("Level: {}", self.level);
        let score_str = format!("Score: {}", self.score);
        let level_display = graphics::Text::new((level_str, self.assets.font, 32.0));
        let score_display = graphics::Text::new((score_str, self.assets.font, 32.0));
        graphics::draw(ctx, &level_display, (level_dest, 0.0, graphics::WHITE))?;
        graphics::draw(ctx, &score_display, (score_dest, 0.0, graphics::WHITE))?;

        // Then we flip the screen...
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
            KeyCode::Up => {
                self.input.yaxis = 1.0;
            }
            KeyCode::Left => {
                self.input.xaxis = -1.0;
            }
            KeyCode::Right => {
                self.input.xaxis = 1.0;
            }
            KeyCode::Space => {
                self.input.fire = true;
            }
            KeyCode::P => {
                let img = graphics::screenshot(ctx).expect("Could not take screenshot");
                img.encode(ctx, graphics::ImageFormat::Png, "/screenshot.png")
                    .expect("Could not save screenshot");
            }
            KeyCode::Escape => ggez::quit(ctx),
            _ => (), // Do nothing
        }
    }

    fn key_up_event(&mut self, _ctx: &mut Context, keycode: KeyCode, _keymod: KeyMods) {
        match keycode {
            KeyCode::Up => {
                self.input.yaxis = 0.0;
            }
            KeyCode::Left | KeyCode::Right => {
                self.input.xaxis = 0.0;
            }
            KeyCode::Space => {
                self.input.fire = false;
            }
            _ => (), // Do nothing
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
        .window_mode(conf::WindowMode::default().dimensions(1024.0, 600.0))
        .add_resource_path(resource_dir);

    let (ctx, events_loop) = &mut cb.build()?;

    let game = &mut MainState::new(ctx)?;
    event::run(ctx, events_loop, game)
}

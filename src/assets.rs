use crate::actor::{Actor, ActorType};

use ggez::{
    audio, graphics, nalgebra as na,
    nalgebra::Vector2,
    {Context, GameResult},
};
use rand::Rng;


pub struct Assets {
    pub player_midflap: graphics::Image,
    pub player_upflap: graphics::Image,
    pub player_downflap: graphics::Image,
    pub pipe_image: graphics::Image,
    pub font: graphics::Font,
    // bg
    pub bg: graphics::Image,
    pub base: graphics::Image,
    //
    pub shot_sound: audio::SpatialSource,
    pub hit_sound: audio::SpatialSource,
    // use up/down flap
}

const COLOR: [&str; 3] = ["red", "yellow", "blue"];

impl Assets {
    pub fn new(ctx: &mut Context) -> GameResult<Assets> {
        // pick a color
        let mut rng = rand::thread_rng();
        let rand_color: usize = rng.gen_range(0, 3);
        let color = COLOR[rand_color];
        // player
        let player_midflap = graphics::Image::new(ctx, format!("/{}bird-midflap.png", color))?;
        let player_upflap = graphics::Image::new(ctx, format!("/{}bird-upflap.png", color))?;
        let player_downflap = graphics::Image::new(ctx, format!("/{}bird-downflap.png", color))?;
        //
        let pipe_image = graphics::Image::new(ctx, "/pipe-green.png")?;
        let font = graphics::Font::new(ctx, "/DejaVuSerif.ttf")?;
        // background
        let bg = graphics::Image::new(ctx, "/background-day.png")?;
        let base = graphics::Image::new(ctx, "/base.png")?;

        let mut shot_sound = audio::SpatialSource::new(ctx, "/pew.ogg")?;
        let mut hit_sound = audio::SpatialSource::new(ctx, "/boom.ogg")?;

        shot_sound.set_ears([-1.0, 0.0, 0.0], [1.0, 0.0, 0.0]);
        hit_sound.set_ears([-1.0, 0.0, 0.0], [1.0, 0.0, 0.0]);

        Ok(Assets {
            player_midflap,
            player_upflap,
            player_downflap,
            pipe_image,
            bg,
            base,
            font,
            shot_sound,
            hit_sound,
        })
    }

    pub fn actor_image(&mut self, actor: &Actor) -> &mut graphics::Image {
        match actor.ty {
            ActorType::Player => {
                // if less than 0 point down
                if actor.velocity.y < na::zero::<Vector2<_>>().y {
                    return &mut self.player_downflap;
                }
                if actor.velocity.y > 1.0 {
                    &mut self.player_upflap
                } else {
                    &mut self.player_midflap
                }
            }
            ActorType::Pipe => &mut self.pipe_image,
        }
    }
}


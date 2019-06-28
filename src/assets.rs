use crate::actor::{ActorType, Actor};

use ggez::audio;
use ggez::graphics;
use ggez::{Context, GameResult};


pub struct Assets {
    pub player_image: graphics::Image,
    pub pipe_image: graphics::Image,
    pub font: graphics::Font,
    // bg
    pub bg: graphics::Image,
    pub base: graphics::Image,
    //
    pub shot_sound: audio::SpatialSource,
    pub hit_sound: audio::SpatialSource,
}

impl Assets {
    pub fn new(ctx: &mut Context) -> GameResult<Assets> {
        let player_image = graphics::Image::new(ctx, "/redbird-midflap.png")?;
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
            player_image,
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
            ActorType::Player => &mut self.player_image,
            ActorType::Pipe => &mut self.pipe_image,
        }
    }
}
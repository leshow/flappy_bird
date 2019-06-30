use crate::actors::Player;

use ggez::{
    audio,
    graphics::{self, spritebatch::SpriteBatch},
    nalgebra as na,
    nalgebra::Vector2,
    {Context, GameResult},
};
use rand::Rng;

pub struct BgAssets {
    pub base: SpriteBatch,
    pub base_w: u16,
    pub base_h: u16,
    pub bg: SpriteBatch,
    pub bg_w: u16,
    pub bg_h: u16,
    pub message: graphics::Image,
    pub pipe_image: graphics::Image,
}

impl BgAssets {
    #[inline]
    pub fn new(ctx: &mut Context) -> GameResult<Self> {
        let message = graphics::Image::new(ctx, "/message.png")?;
        // background
        let bg_img = graphics::Image::new(ctx, "/background-day.png")?;
        let bg_w = bg_img.width();
        let bg_h = bg_img.height();
        let bg = SpriteBatch::new(bg_img);
        //
        let base_img = graphics::Image::new(ctx, "/base.png")?;
        let base_w = base_img.width();
        let base_h = base_img.height();
        let base = SpriteBatch::new(base_img);
        // let pipe_image = SpriteBatch::new(graphics::Image::new(ctx, "/pipe-green.png")?);
        let pipe_image = graphics::Image::new(ctx, "/pipe-green.png")?;

        Ok(BgAssets {
            base,
            base_w,
            base_h,
            bg,
            bg_w,
            bg_h,
            pipe_image,
            message,
        })
    }
}

pub struct PlayerAssets {
    pub player_midflap: graphics::Image,
    pub player_upflap: graphics::Image,
    pub player_downflap: graphics::Image,
}

impl PlayerAssets {
    #[inline]
    pub fn new(ctx: &mut Context) -> GameResult<Self> {
        // pick a color
        let mut rng = rand::thread_rng();
        let rand_color: usize = rng.gen_range(0, 3);
        let color = PlayerAssets::color()[rand_color];
        //
        let player_midflap = graphics::Image::new(ctx, format!("/{}bird-midflap.png", color))?;
        let player_upflap = graphics::Image::new(ctx, format!("/{}bird-upflap.png", color))?;
        let player_downflap = graphics::Image::new(ctx, format!("/{}bird-downflap.png", color))?;

        Ok(PlayerAssets {
            player_midflap,
            player_upflap,
            player_downflap,
        })
    }

    #[inline]
    pub const fn color() -> [&'static str; 3] {
        ["red", "yellow", "blue"]
    }
}

pub struct Assets {
    pub player: PlayerAssets,
    pub font: graphics::Font,
    // bg
    pub bg: BgAssets,
    //
    pub shot_sound: audio::SpatialSource,
    pub hit_sound: audio::SpatialSource,
    // use up/down flap
}

impl Assets {
    pub fn new(ctx: &mut Context) -> GameResult<Self> {
        //
        let font = graphics::Font::new(ctx, "/DejaVuSerif.ttf")?;

        let mut shot_sound = audio::SpatialSource::new(ctx, "/pew.ogg")?;
        let mut hit_sound = audio::SpatialSource::new(ctx, "/boom.ogg")?;

        shot_sound.set_ears([-1., 0., 0.], [1., 0., 0.]);
        hit_sound.set_ears([-1., 0., 0.], [1., 0., 0.]);

        Ok(Assets {
            player: PlayerAssets::new(ctx)?,
            font,
            shot_sound,
            hit_sound,
            bg: BgAssets::new(ctx)?,
        })
    }

    pub fn player_image(&mut self, actor: &Player) -> &mut graphics::Image {
        // if less than 0 point down
        if actor.velocity.y < na::zero::<Vector2<_>>().y {
            return &mut self.player.player_downflap;
        }
        if actor.velocity.y > 1. {
            &mut self.player.player_upflap
        } else {
            &mut self.player.player_midflap
        }
    }
}


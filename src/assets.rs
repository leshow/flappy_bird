use crate::actors::Player;

use ggez::{
    audio,
    graphics::{self, spritebatch::SpriteBatch},
    {Context, GameResult},
};
use rand::Rng;

#[derive(Debug, Clone)]
pub struct BgAssets {
    pub base: SpriteBatch,
    pub base_w: u16,
    pub base_h: u16,
    pub bg: SpriteBatch,
    pub bg_w: u16,
    pub bg_h: u16,
    pub pipe_img: graphics::Image,
    pub pipe: SpriteBatch,
}

impl BgAssets {
    #[inline]
    pub fn new<R>(ctx: &mut Context, rng: &mut R) -> GameResult<Self>
    where
        R: Rng + ?Sized,
    {
        let rand_style: usize = rng.gen_range(0, 2);
        let style = BgAssets::style()[rand_style];
        // background
        let bg_img = graphics::Image::new(ctx, format!("/background-{}.png", style))?;
        let bg_w = bg_img.width();
        let bg_h = bg_img.height();
        let bg = SpriteBatch::new(bg_img);
        //
        let base_img = graphics::Image::new(ctx, "/base.png")?;
        let base_w = base_img.width();
        let base_h = base_img.height();
        let base = SpriteBatch::new(base_img);
        let pipe_img = graphics::Image::new(ctx, "/pipe-green.png")?;
        let pipe = SpriteBatch::new(pipe_img.clone());

        Ok(BgAssets {
            base,
            base_w,
            base_h,
            bg,
            bg_w,
            bg_h,
            pipe_img,
            pipe,
        })
    }

    #[inline]
    pub const fn style() -> [&'static str; 2] {
        ["day", "night"]
    }
}

#[derive(Debug, Clone)]
pub struct PlayerAssets {
    pub player_midflap: graphics::Image,
    pub player_upflap: graphics::Image,
    pub player_downflap: graphics::Image,
}

impl PlayerAssets {
    #[inline]
    pub fn new<R>(ctx: &mut Context, rng: &mut R) -> GameResult<Self>
    where
        R: Rng + ?Sized,
    {
        let rand_color: usize = rng.gen_range(0, 2);
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

#[derive(Debug )]
pub struct Assets {
    pub player: PlayerAssets,
    pub font: graphics::Font,
    pub gameover: graphics::Image,
    pub message: graphics::Image,
    // bg
    pub bg: BgAssets,
    //
    pub shot_sound: audio::SpatialSource,
    pub hit_sound: audio::SpatialSource,
    // use up/down flap
}

impl Assets {
    pub fn new(ctx: &mut Context) -> GameResult<Self> {
        let mut rng = rand::thread_rng();
        //
        let font = graphics::Font::new(ctx, "/DejaVuSerif.ttf")?;

        let gameover = graphics::Image::new(ctx, "/gameover.png")?;
        let message = graphics::Image::new(ctx, "/message.png")?;
        let mut shot_sound = audio::SpatialSource::new(ctx, "/pew.ogg")?;
        let mut hit_sound = audio::SpatialSource::new(ctx, "/boom.ogg")?;

        shot_sound.set_ears([-1., 0., 0.], [1., 0., 0.]);
        hit_sound.set_ears([-1., 0., 0.], [1., 0., 0.]);

        Ok(Assets {
            player: PlayerAssets::new(ctx, &mut rng)?,
            font,
            message,
            gameover,
            shot_sound,
            hit_sound,
            bg: BgAssets::new(ctx, &mut rng)?,
        })
    }

    pub fn player_image(&mut self, actor: &Player, frames: u64) -> &mut graphics::Image {
        // if less than 0 point down
        if actor.velocity.y < -3. {
            return &mut self.player.player_downflap;
        }
        if frames <= 5 {
            &mut self.player.player_upflap
        } else if frames <= 10 {
            &mut self.player.player_midflap
        } else {
            &mut self.player.player_downflap
        }
    }
}

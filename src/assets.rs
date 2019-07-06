use crate::actors::Player;

use ggez::{
    audio,
    graphics::{self, spritebatch::SpriteBatch, Image},
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
    pub pipe_img: Image,
    pub pipe: SpriteBatch,
    pub numbers: [Image; 10],
}

impl BgAssets {
    #[inline]
    pub fn new<R>(ctx: &mut Context, rng: &mut R) -> GameResult<Self>
    where
        R: Rng + ?Sized,
    {
        let style = BgAssets::style()[rng.gen_range(0, 2)];
        let pipe_color = BgAssets::pipe_color()[rng.gen_range(0, 2)];
        // background
        let bg_img = Image::new(ctx, format!("/background-{}.png", style))?;
        let bg_w = bg_img.width();
        let bg_h = bg_img.height();
        let bg = SpriteBatch::new(bg_img);
        //
        let base_img = Image::new(ctx, "/base.png")?;
        let base_w = base_img.width();
        let base_h = base_img.height();
        let base = SpriteBatch::new(base_img);
        let pipe_img = Image::new(ctx, format!("/pipe-{}.png", pipe_color))?;
        let pipe = SpriteBatch::new(pipe_img.clone());
        // numbers for countdown
        let numbers = [
            Image::new(ctx, "/0.png")?,
            Image::new(ctx, "/1.png")?,
            Image::new(ctx, "/2.png")?,
            Image::new(ctx, "/3.png")?,
            Image::new(ctx, "/4.png")?,
            Image::new(ctx, "/5.png")?,
            Image::new(ctx, "/6.png")?,
            Image::new(ctx, "/7.png")?,
            Image::new(ctx, "/8.png")?,
            Image::new(ctx, "/9.png")?,
        ];

        Ok(BgAssets {
            base,
            base_w,
            base_h,
            bg,
            bg_w,
            bg_h,
            pipe_img,
            pipe,
            numbers,
        })
    }

    #[inline]
    pub const fn style() -> [&'static str; 2] {
        ["day", "night"]
    }

    #[inline]
    pub const fn pipe_color() -> [&'static str; 2] {
        ["green", "red"]
    }
}

#[derive(Debug, Clone)]
pub struct PlayerAssets {
    pub player_midflap: Image,
    pub player_upflap: Image,
    pub player_downflap: Image,
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
        let player_midflap = Image::new(ctx, format!("/{}bird-midflap.png", color))?;
        let player_upflap = Image::new(ctx, format!("/{}bird-upflap.png", color))?;
        let player_downflap = Image::new(ctx, format!("/{}bird-downflap.png", color))?;

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

#[derive(Debug)]
pub struct Assets {
    pub player: PlayerAssets,
    pub font: graphics::Font,
    pub gameover: Image,
    pub message: Image,
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

        let gameover = Image::new(ctx, "/gameover.png")?;
        let message = Image::new(ctx, "/message.png")?;
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

    pub fn player_image(&mut self, actor: &Player, frames: u64) -> &mut Image {
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

    pub fn countdown(&mut self, frames: u64) -> Option<&mut Image> {
        if frames <= 60 {
            Some(&mut self.bg.numbers[3])
        } else if frames <= 120 {
            Some(&mut self.bg.numbers[2])
        } else if frames <= 180 {
            Some(&mut self.bg.numbers[1])
        } else {
            None
        }
    }
}

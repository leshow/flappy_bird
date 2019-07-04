use crate::{assets::Assets, util::vec_from_angle};

use ggez::{
    nalgebra as na,
    nalgebra::{Point2, Vector2},
};
use rand::Rng;

// bit of a useless trait right now
pub trait Actor {
    fn new() -> Self;
    fn update_pos(&mut self, dt: f32);
}

#[derive(Debug, PartialEq, PartialOrd)]
pub struct Player {
    pub pos: Point2<f32>,
    pub velocity: Vector2<f32>,
    pub facing: f32,
    pub bbox_size: Point2<f32>,
}

#[derive(Debug, PartialEq, PartialOrd)]
pub struct Pipe {
    pub pos: Point2<f32>,
    pub facing: f32,
    pub bbox_size: Point2<f32>,
}

impl Player {
    pub const UP_ANGLE_MAX: f32 = -0.45;
    pub const DOWN_ANGLE_MAX: f32 = 1.5;

    pub fn flap(&mut self, dt: f32) {
        let dir = vec_from_angle(0.);
        let flap_vec = dir * crate::FLAP_SPEED;
        // set constant velocity on flap
        self.velocity = flap_vec * dt;
        // makes for more "real" physics but is not flappy bird:
        // player.velocity += flap_vec * dt;
        self.facing = Player::UP_ANGLE_MAX;
    }
}

impl Actor for Player {
    fn new() -> Self {
        Player {
            pos: Point2::origin(),
            velocity: na::zero(),
            bbox_size: Point2::new(14., 12.),
            facing: 0.,
        }
    }

    fn update_pos(&mut self, dt: f32) {
        let dir = vec_from_angle(0.);
        let grav = dir * crate::FALL_SPEED;
        self.velocity -= grav * dt;
        self.pos += self.velocity;
        // set dir bird is facing
        self.facing -= self.velocity.y * dt;
        if self.facing >= Player::DOWN_ANGLE_MAX {
            self.facing = Player::DOWN_ANGLE_MAX;
        } else if self.facing <= Player::UP_ANGLE_MAX {
            self.facing = Player::UP_ANGLE_MAX;
        }
    }
}

impl Default for Player {
    fn default() -> Self {
        Player::new()
    }
}

impl Default for Pipe {
    fn default() -> Self {
        Pipe::new()
    }
}

impl Pipe {
    pub const PIPE_GAP: f32 = 40.;
    pub const BETWEEN_PIPE: f32 = 300.;
    pub const FIRST_PIPE_X: f32 = 200.;
    pub const MIN_RANGE: f32 = 155.;
}

impl Actor for Pipe {
    fn new() -> Self {
        Pipe {
            pos: Point2::origin(),
            facing: 0.,
            bbox_size: Point2::new(26., 160.),
        }
    }

    fn update_pos(&mut self, _dt: f32) {
        unimplemented!();
    }
}

pub fn gen_pipes(assets: &Assets, screen_width: f32) -> Vec<(Pipe, Pipe)> {
    let height = f32::from(assets.bg.bg_h);
    let pipe_h = f32::from(assets.bg.pipe_img.height()) / 2.;
    let first_pipe = Point2::new((screen_width / 2.) + Pipe::FIRST_PIPE_X, height - pipe_h);
    let mut rng = rand::thread_rng();

    (1..=10)
        .map(|i| {
            let new_x = i as f32 * Pipe::BETWEEN_PIPE;
            let opening: f32 = rng.gen_range(Pipe::MIN_RANGE, height - Pipe::MIN_RANGE);
            // bottom pipe
            let mut bottom_pipe = Pipe::new();
            let pos = Point2::new(first_pipe.x + new_x, opening + Pipe::PIPE_GAP + pipe_h);
            bottom_pipe.pos = pos;
            // top pipe
            let mut top_pipe = Pipe::new();
            top_pipe.pos = Point2::new(first_pipe.x + new_x, opening - Pipe::PIPE_GAP - pipe_h);
            top_pipe.facing = std::f32::consts::PI;

            (bottom_pipe, top_pipe)
        })
        .collect()
}

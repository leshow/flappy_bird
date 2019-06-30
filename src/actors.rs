use crate::util::vec_from_angle;

use ggez::{
    nalgebra as na,
    nalgebra::{Point2, Vector2},
};

pub trait Actor {
    fn new() -> Self;
    fn update_pos(&mut self, dt: f32);
}

#[derive(Debug, PartialEq, PartialOrd)]
pub struct Player {
    pub pos: Point2<f32>,
    pub velocity: Vector2<f32>,
    pub facing: f32,
    pub bbox_size: f32,
}

#[derive(Debug, PartialEq, PartialOrd)]
pub struct Pipe {
    pub pos: Point2<f32>,
    pub velocity: Vector2<f32>,
    pub bbox_size: f32,
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
            bbox_size: crate::PLAYER_BBOX,
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
    pub fn new() -> Self {
        Pipe {
            pos: Point2::origin(),
            velocity: na::zero(),
            bbox_size: crate::PIPE_BBOX,
        }
    }
}

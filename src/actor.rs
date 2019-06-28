use ggez::{
    nalgebra as na,
    nalgebra::{Point2, Vector2},
};

#[derive(Debug, PartialEq, PartialOrd, Ord, Eq)]
pub enum ActorType {
    Player,
    Pipe,
}

#[derive(Debug, PartialEq, PartialOrd)]
pub struct Actor {
    pub ty: ActorType,
    pub pos: Point2<f32>,
    pub velocity: Vector2<f32>,
    pub bbox_size: f32,
}

impl Actor {
    pub fn new(ty: ActorType) -> Self {
        match ty {
            ActorType::Player => Actor {
                ty,
                pos: Point2::origin(),
                velocity: na::zero(),
                bbox_size: crate::PLAYER_BBOX,
            },
            ActorType::Pipe => Actor {
                ty,
                pos: Point2::origin(),
                velocity: na::zero(),
                bbox_size: crate::PIPE_BBOX,
            },
        }
    }
}

pub fn player_flap(player: &mut Actor, dt: f32) {
    let dir = vec_from_angle(0.0);
    let flap_vec = dir * crate::FLAP_SPEED;
    player.velocity += flap_vec * dt;
}

pub fn update_player_pos(player: &mut Actor, dt: f32) {
    let dv = player.velocity * dt;
    player.pos += dv;
}

pub fn player_gravity(player: &mut Actor, dt: f32) {
    let dir = vec_from_angle(0.0); // should be (0, 1)
    let grav_vec = dir * crate::FALL_SPEED;
    player.velocity += grav_vec * dt;
}

pub fn vec_from_angle(angle: f32) -> Vector2<f32> {
    let vx = angle.sin();
    let vy = angle.cos();
    Vector2::new(vx, vy)
}

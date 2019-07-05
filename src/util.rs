use ggez::nalgebra::{Point2, Vector2};

pub fn vec_from_angle(angle: f32) -> Vector2<f32> {
    let vx = angle.sin();
    let vy = angle.cos();
    Vector2::new(vx, vy)
}

/// Translates the world coordinate system, which
/// has Y pointing up and the origin at the center,
/// to the screen coordinate system, which has Y
/// pointing downward and the origin at the top-left,
pub fn translate_coords(point: Point2<f32>, screen_width: f32, screen_height: f32) -> Point2<f32> {
    let x = point.x + screen_width / 2.;
    let y = screen_height - (point.y + screen_height / 2.);
    Point2::new(x, y)
}
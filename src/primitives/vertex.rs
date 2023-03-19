use super::color::Color;
use nalgebra::Point3;

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct ColoredVertex {
    position: Point3<f32>,
    color: Color,
}

impl ColoredVertex {
    pub fn new(x: f32, y: f32, z: f32, r: f32, g: f32, b: f32) -> ColoredVertex {
        ColoredVertex {
            position: Point3::new(x, y, z),
            color: Color::new(r, g, b),
        }
    }
}

use nalgebra::Point3;

pub trait Gridable {
    fn grid(&self, points_x: u32, points_y: u32) -> (Vec<Point3<f32>>, Vec<(u32, u32)>);
}

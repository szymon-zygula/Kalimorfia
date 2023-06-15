use crate::render::mesh::SurfaceVertex;

pub trait Gridable {
    fn grid(&self, points_x: u32, points_y: u32) -> (Vec<SurfaceVertex>, Vec<u32>);
}

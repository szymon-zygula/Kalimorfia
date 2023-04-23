use nalgebra::Point3;

pub fn point_64_to_32(p: Point3<f64>) -> Point3<f32>
where
{
    Point3::new(p.x as f32, p.y as f32, p.z as f32)
}

pub fn point_32_to_64(p: Point3<f32>) -> Point3<f64>
where
{
    Point3::new(p.x as f64, p.y as f64, p.z as f64)
}

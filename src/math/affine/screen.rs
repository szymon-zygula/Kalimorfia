use glutin::dpi::{PhysicalPosition, PhysicalSize};
use nalgebra::{Point2, Vector2};

pub fn screen_to_ndc(
    resolution: &PhysicalSize<u32>,
    position: &PhysicalPosition<u32>,
) -> Point2<f32> {
    2.0 * Point2::new(
        position.x as f32 / resolution.width as f32 - 0.5,
        0.5 - position.y as f32 / resolution.height as f32,
    )
}

pub fn ndc_to_screen(
    resolution: &PhysicalSize<u32>,
    coordinates: &Point2<f32>,
) -> PhysicalPosition<u32> {
    let coordinates = (coordinates + Vector2::new(1.0, 1.0)) / 2.0;

    let x = (coordinates.x * resolution.width as f32).round() as u32;
    let y = ((1.0 - coordinates.y) * resolution.height as f32).round() as u32;

    PhysicalPosition::new(
        x.clamp(0, resolution.width - 1),
        y.clamp(0, resolution.height - 1),
    )
}

pub fn clamp_screen(
    resolution: &PhysicalSize<u32>,
    position: &PhysicalPosition<u32>,
) -> PhysicalPosition<u32> {
    PhysicalPosition::new(
        position.x.clamp(0, resolution.width - 1),
        position.y.clamp(0, resolution.height - 1),
    )
}

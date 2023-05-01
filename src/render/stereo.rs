use crate::camera::Camera;
use glow::HasContext;

pub fn draw(
    gl: &glow::Context,
    left_camera: &Camera,
    right_camera: &Camera,
    mut draw: impl FnMut(&Camera),
) {
    unsafe { gl.color_mask(true, false, false, true) };
    draw(left_camera);
    unsafe { gl.color_mask(false, true, true, true) };
    draw(right_camera);
    unsafe { gl.color_mask(true, true, true, true) };
}

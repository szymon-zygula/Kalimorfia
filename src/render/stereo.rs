use crate::{
    camera::Camera,
    constants::{CLEAR_COLOR, STEREO_CLEAR_COLOR},
};
use glow::HasContext;

pub fn draw(
    gl: &glow::Context,
    left_camera: &Camera,
    right_camera: &Camera,
    mut draw: impl FnMut(&Camera),
) {
    unsafe {
        gl.clear_color(
            STEREO_CLEAR_COLOR.r,
            STEREO_CLEAR_COLOR.g,
            STEREO_CLEAR_COLOR.b,
            STEREO_CLEAR_COLOR.a,
        )
    };

    unsafe { gl.color_mask(true, false, false, true) };
    draw(right_camera);
    unsafe { gl.color_mask(false, true, true, true) };
    draw(left_camera);
    unsafe { gl.color_mask(true, true, true, true) };

    unsafe { gl.clear_color(CLEAR_COLOR.r, CLEAR_COLOR.g, CLEAR_COLOR.b, CLEAR_COLOR.a) };
}

use crate::primitives::color::ColorAlpha;

pub const WINDOW_TITLE: &str = "Kalimorfia";
pub const WINDOW_WIDTH: u32 = 1280;
pub const WINDOW_HEIGHT: u32 = 720;
pub const CLEAR_COLOR: ColorAlpha = ColorAlpha {
    r: 0.4,
    g: 0.4,
    b: 0.4,
    a: 1.0,
};
pub const STEREO_CLEAR_COLOR: ColorAlpha = ColorAlpha {
    r: 0.0,
    g: 0.0,
    b: 0.0,
    a: 1.0,
};

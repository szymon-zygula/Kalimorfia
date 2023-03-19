#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
}

impl Color {
    pub fn new(r: f32, g: f32, b: f32) -> Color {
        Color { r, g, b }
    }
}

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct ColorAlpha {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl ColorAlpha {
    pub fn new(r: f32, g: f32, b: f32, a: f32) -> ColorAlpha {
        ColorAlpha { r, g, b, a }
    }
}

use crate::entities::entity::DrawType;

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
}

impl Color {
    pub fn new(r: f32, g: f32, b: f32) -> Self {
        Color { r, g, b }
    }

    pub fn white() -> Self {
        Self::new(1.0, 1.0, 1.0)
    }

    pub fn orange() -> Self {
        Self::new(1.0, 0.5, 0.0)
    }

    pub fn purple() -> Self {
        Self::new(1.0, 0.0, 0.5)
    }

    pub fn green() -> Self {
        Self::new(0.0, 1.0, 0.0)
    }

    pub fn for_draw_type(draw_type: &DrawType) -> Self {
        match draw_type {
            DrawType::Regular => Self::white(),
            DrawType::Virtual => Self::purple(),
            DrawType::Selected => Self::orange(),
            DrawType::SelectedVirtual => Self::green(),
        }
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
    pub fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }
}

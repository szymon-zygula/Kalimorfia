#[derive(Copy, Clone, Debug)]
pub struct BezierFlatSurfaceArgs {
    pub x_length: f32,
    pub z_length: f32,

    pub x_patches: i32,
    pub z_patches: i32,
}

#[derive(Copy, Clone, Debug)]
pub struct BezierCylinderArgs {
    pub length: f32,
    pub radius: f32,

    pub around_patches: i32,
    pub along_patches: i32,
}

#[derive(Copy, Clone, Debug)]
pub enum BezierSurfaceArgs {
    Surface(BezierFlatSurfaceArgs),
    Cylinder(BezierCylinderArgs),
}

impl BezierSurfaceArgs {
    const MIN_PATCHES: i32 = 1;
    const MAX_PATCHES: i32 = 30;
    const MIN_LENGTH: f32 = 0.1;
    const MAX_LENGTH: f32 = 10.0;

    pub fn new_surface() -> Self {
        Self::Surface(BezierFlatSurfaceArgs {
            x_length: 1.0,
            z_length: 1.0,

            x_patches: 1,
            z_patches: 1,
        })
    }

    pub fn new_cylinder() -> Self {
        Self::Cylinder(BezierCylinderArgs {
            length: 1.0,
            radius: 1.0,
            around_patches: 1,
            along_patches: 1,
        })
    }

    pub fn clamp_values(&mut self) {
        match self {
            BezierSurfaceArgs::Surface(surface) => {
                Self::clamp_patches(&mut surface.x_patches);
                Self::clamp_patches(&mut surface.z_patches);
                Self::clamp_length(&mut surface.x_length);
                Self::clamp_length(&mut surface.z_length);
            }
            BezierSurfaceArgs::Cylinder(cyllinder) => {
                Self::clamp_patches(&mut cyllinder.around_patches);
                Self::clamp_patches(&mut cyllinder.along_patches);
                Self::clamp_length(&mut cyllinder.length);
                Self::clamp_length(&mut cyllinder.radius);
            }
        }
    }

    fn clamp_patches(patches: &mut i32) {
        if *patches < Self::MIN_PATCHES {
            *patches = Self::MIN_PATCHES;
        } else if *patches > Self::MAX_PATCHES {
            *patches = Self::MAX_PATCHES;
        }
    }

    fn clamp_length(length: &mut f32) {
        if *length < Self::MIN_LENGTH {
            *length = Self::MIN_LENGTH;
        } else if *length > Self::MAX_LENGTH {
            *length = Self::MAX_LENGTH;
        }
    }
}

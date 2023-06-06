use nalgebra::{Point3, Vector3};

#[derive(Clone, Debug)]
#[repr(C)]
pub struct GregoryPatch {
    pub top: [Point3<f32>; 4],
    pub top_sides: [Point3<f32>; 2],
    pub bottom_sides: [Point3<f32>; 2],
    pub bottom: [Point3<f32>; 4],
    pub u_inner: [Point3<f32>; 4],
    pub v_inner: [Point3<f32>; 4],
}

pub struct GregoryTriangle {
    patches: [GregoryPatch; 3],
}

pub struct BorderPatch([[Point3<f32>; 4]; 4]);

impl BorderPatch {
    pub fn border_points(&self) -> [[Point3<f32>; 4]; 2] {
        todo!()
    }

    pub fn border_tangents(&self) -> [[Vector3<f32>; 4]; 2] {
        todo!()
    }
}

impl GregoryTriangle {
    /// `border_patches` are assumed to be orderded in the same way as in `graph::C0EdgeGraph`
    pub fn new(border_patches: [BorderPatch; 3]) -> Self {
        let border_points: Vec<_> = border_patches.iter().map(|p| p.border_points()).collect();
        let border_tangents: Vec<_> = border_patches.iter().map(|p| p.border_tangents()).collect();

        let p30 = border_points[0][1][0];
        let p31 = border_points[1][1][0];
        let p32 = border_points[2][1][0];

        let p20 = p30 + border_tangents[0][1][0];
        let p21 = p31 + border_tangents[1][1][0];
        let p22 = p32 + border_tangents[2][1][0];

        todo!()
    }
}

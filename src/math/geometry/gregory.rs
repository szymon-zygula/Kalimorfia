use crate::math::{
    geometry::{bezier::BezierCurve, parametric_form::ParametricForm},
    utils,
};
use nalgebra::{Point3, Vector1, Vector3};

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

pub struct BorderPatch(pub [[Point3<f32>; 4]; 4]);

impl BorderPatch {
    pub fn points(&self) -> [[Point3<f32>; 4]; 2] {
        let bernstein = BezierCurve::through_points(
            &self.0[0]
                .iter()
                .copied()
                .map(utils::point_32_to_64)
                .collect::<Vec<_>>(),
        );

        let divided = bernstein.divide_at(0.5);
        let divided0: Vec<_> = divided
            .0
            .points()
            .iter()
            .copied()
            .map(utils::point_64_to_32)
            .collect();

        let divided1: Vec<_> = divided
            .1
            .points()
            .iter()
            .copied()
            .map(utils::point_64_to_32)
            .collect();

        [
            [divided0[0], divided0[1], divided0[2], divided0[3]],
            [divided1[0], divided1[1], divided1[2], divided1[3]],
        ]
    }

    pub fn diff_u(&self) -> [Vector3<f32>; 3] {
        let bezier0 = BezierCurve::through_points(&[
            utils::point_32_to_64(self.0[0][0]),
            utils::point_32_to_64(self.0[1][0]),
            utils::point_32_to_64(self.0[2][0]),
            utils::point_32_to_64(self.0[3][0]),
        ]);

        let bezier1 = BezierCurve::through_points(&[
            utils::point_32_to_64(self.0[0][3]),
            utils::point_32_to_64(self.0[1][3]),
            utils::point_32_to_64(self.0[2][3]),
            utils::point_32_to_64(self.0[3][3]),
        ]);

        let bezier_front = BezierCurve::through_points(&[
            utils::point_32_to_64(self.0[0][0]),
            utils::point_32_to_64(self.0[0][1]),
            utils::point_32_to_64(self.0[0][2]),
            utils::point_32_to_64(self.0[0][3]),
        ]);

        let bezier_back = BezierCurve::through_points(&[
            utils::point_32_to_64(self.0[1][0]),
            utils::point_32_to_64(self.0[1][1]),
            utils::point_32_to_64(self.0[1][2]),
            utils::point_32_to_64(self.0[1][3]),
        ]);

        // https://www.rose-hulman.edu/~finn/CCLI/Notes/day27.pdf
        let front_val = bezier_front.parametric(&Vector1::new(0.5));
        let back_val = bezier_back.parametric(&Vector1::new(0.5));

        [
            utils::vec_64_to_32(bezier0.derivative(0.0)),
            3.0 * utils::vec_64_to_32(back_val - front_val),
            utils::vec_64_to_32(bezier1.derivative(0.0)),
        ]
    }

    pub fn twist(&self) -> [Vector3<f32>; 3] {
        let p = &self.0;
        // https://www.rose-hulman.edu/~finn/CCLI/Notes/day27.pdf
        let mut coeffs = Vec::new();
        for j in 0..3 {
            coeffs.push(Point3::from(utils::vec_32_to_64(
                9.0 * (p[1][j + 1].coords - p[0][j + 1].coords - p[1][j].coords + p[0][j].coords),
            )));
        }

        let twist_bezier = BezierCurve::through_points(&coeffs);
        let w0 = utils::vec_64_to_32(twist_bezier.parametric(&Vector1::new(0.0)).coords);
        let w1 = utils::vec_64_to_32(twist_bezier.parametric(&Vector1::new(0.5)).coords);
        let w2 = utils::vec_64_to_32(twist_bezier.parametric(&Vector1::new(1.0)).coords);

        [w0, w1, w2]
    }

    pub fn subdivide(&self) -> (BezierCurve, BezierCurve) {
        let points = self.points();

        let bezier0 = BezierCurve::through_points(&[
            utils::point_32_to_64(points[0][0]),
            utils::point_32_to_64(points[0][1]),
            utils::point_32_to_64(points[0][2]),
            utils::point_32_to_64(points[0][3]),
        ]);

        let bezier1 = BezierCurve::through_points(&[
            utils::point_32_to_64(points[1][0]),
            utils::point_32_to_64(points[1][1]),
            utils::point_32_to_64(points[1][2]),
            utils::point_32_to_64(points[1][3]),
        ]);

        (bezier0, bezier1)
    }

    pub fn diff_v(&self) -> [[Vector3<f32>; 4]; 2] {
        let bezier = self.subdivide();

        [
            [
                utils::vec_64_to_32(bezier.0.derivative(0.0)),
                utils::vec_64_to_32(bezier.0.derivative(1.0 / 3.0)),
                utils::vec_64_to_32(bezier.0.derivative(2.0 / 3.0)),
                utils::vec_64_to_32(bezier.0.derivative(1.0)),
            ],
            [
                utils::vec_64_to_32(bezier.1.derivative(0.0)),
                utils::vec_64_to_32(bezier.1.derivative(1.0 / 3.0)),
                utils::vec_64_to_32(bezier.1.derivative(2.0 / 3.0)),
                utils::vec_64_to_32(bezier.1.derivative(1.0)),
            ],
        ]
    }

    pub fn points_v(&self) -> [[Point3<f32>; 4]; 2] {
        let bezier = self.subdivide();

        [
            [
                utils::point_64_to_32(bezier.0.parametric(&Vector1::new(0.0))),
                utils::point_64_to_32(bezier.0.parametric(&Vector1::new(1.0 / 3.0))),
                utils::point_64_to_32(bezier.0.parametric(&Vector1::new(2.0 / 3.0))),
                utils::point_64_to_32(bezier.0.parametric(&Vector1::new(1.0))),
            ],
            [
                utils::point_64_to_32(bezier.1.parametric(&Vector1::new(0.0))),
                utils::point_64_to_32(bezier.1.parametric(&Vector1::new(1.0 / 3.0))),
                utils::point_64_to_32(bezier.1.parametric(&Vector1::new(2.0 / 3.0))),
                utils::point_64_to_32(bezier.1.parametric(&Vector1::new(1.0))),
            ],
        ]
    }
}

pub struct GregoryTriangle {
    pub patches: [GregoryPatch; 3],
    // indexed as [patch][subpatch][point]
    pub v_diff: [[[Vector3<f32>; 4]; 2]; 3],
    pub v_diff_p: [[[Point3<f32>; 4]; 2]; 3],
    // indexed as [patch][point]
    pub u_diff: [[Vector3<f32>; 3]; 3],
    pub twist: [[Vector3<f32>; 3]; 3],
    pub twist_u_p: [[Point3<f32>; 3]; 3],
}

impl GregoryTriangle {
    /// `border_patches` are assumed to be orderded in the same way as in `graph::C0EdgeGraph`
    pub fn new(border_patches: [BorderPatch; 3]) -> Self {
        let border_points: Vec<_> = border_patches.iter().map(|p| p.points()).collect();
        let border_tangents: Vec<_> = border_patches.iter().map(|p| p.diff_u()).collect();

        let p30 = border_points[0][1][0];
        let p31 = border_points[1][1][0];
        let p32 = border_points[2][1][0];

        let p20 = p30 - border_tangents[0][1] / 3.0;
        let p21 = p31 - border_tangents[1][1] / 3.0;
        let p22 = p32 - border_tangents[2][1] / 3.0;

        let q0 = (3.0 * p20 - p30) / 2.0;
        let q1 = (3.0 * p21 - p31) / 2.0;
        let q2 = (3.0 * p22 - p32) / 2.0;

        let p = Point3::from((q0 + q1 + q2) / 3.0);

        let p10 = (p + 2.0 * q0) / 3.0;
        let p11 = (p + 2.0 * q1) / 3.0;
        let p12 = (p + 2.0 * q2) / 3.0;

        let [points00, points10] = border_patches[0].points();
        let [points01, points11] = border_patches[1].points();
        let [points02, points12] = border_patches[2].points();

        let u0 = border_patches[0].diff_u();
        let u1 = border_patches[1].diff_u();
        let u2 = border_patches[2].diff_u();

        let [v00, v10] = border_patches[0].diff_v();
        let [v01, v11] = border_patches[1].diff_v();
        let [v02, v12] = border_patches[2].diff_v();

        let w0 = border_patches[0].twist();
        let w1 = border_patches[1].twist();
        let w2 = border_patches[2].twist();

        Self {
            twist: [w0, w1, w2],
            twist_u_p: [
                [points00[0], p30, points10[3]],
                [points01[0], p31, points11[3]],
                [points02[0], p32, points12[3]],
            ],
            u_diff: [u0, u1, u2],
            v_diff: [[v00, v10], [v01, v11], [v02, v12]],
            v_diff_p: [
                border_patches[0].points_v(),
                border_patches[1].points_v(),
                border_patches[2].points_v(),
            ],
            patches: [
                GregoryPatch {
                    top: [p, p10, p20, p30],
                    top_sides: [p12, points00[2]],
                    bottom_sides: [p22, points00[1]],
                    bottom: points12,
                    u_inner: [
                        p + (p12 - p) + (p10 - p),
                        p30 - u0[1] / 3.0 - v00[3] / 3.0 + w0[1] / 9.0,
                        points12[0] - u2[1] / 3.0 + v12[0] / 3.0 + w2[1] / 9.0,
                        points12[3] - u2[2] / 3.0 - v12[3] / 3.0 + w2[2] / 9.0,
                    ],
                    v_inner: [
                        p + (p12 - p) + (p10 - p),
                        p30 - u0[1] / 3.0 - v00[3] / 3.0 + w0[1] / 9.0,
                        points12[0] - u2[1] / 3.0 + v12[0] / 3.0 + w2[1] / 9.0,
                        points00[0] - u0[0] / 3.0 + v00[0] / 3.0 + w0[0] / 9.0,
                    ],
                },
                GregoryPatch {
                    top: [p, p11, p21, p31],
                    top_sides: [p10, points01[2]],
                    bottom_sides: [p20, points01[1]],
                    bottom: points10,
                    u_inner: [
                        p + (p10 - p) + (p11 - p),
                        p31 - u1[1] / 3.0 - v01[3] / 3.0 + w1[1] / 9.0,
                        points10[0] - u0[1] / 3.0 + v10[0] / 3.0 + w0[1] / 9.0,
                        points10[3] - u0[2] / 3.0 - v10[3] / 3.0 + w0[2] / 9.0,
                    ],
                    v_inner: [
                        p + (p10 - p) + (p11 - p),
                        p31 - u1[1] / 3.0 - v01[3] / 3.0 + w1[1] / 9.0,
                        points10[0] - u0[1] / 3.0 + v10[0] / 3.0 + w0[1] / 9.0,
                        points01[0] - u1[0] / 3.0 + v01[0] / 3.0 + w1[0] / 9.0,
                    ],
                },
                GregoryPatch {
                    top: [p, p12, p22, p32],
                    top_sides: [p11, points02[2]],
                    bottom_sides: [p21, points02[1]],
                    bottom: points11,
                    u_inner: [
                        p + (p11 - p) + (p12 - p),
                        p32 - u2[1] / 3.0 - v02[3] / 3.0 + w2[1] / 9.0,
                        points11[0] - u1[1] / 3.0 + v11[0] / 3.0 + w1[1] / 9.0,
                        points11[3] - u1[2] / 3.0 - v11[3] / 3.0 + w1[2] / 9.0,
                    ],
                    v_inner: [
                        p + (p11 - p) + (p12 - p),
                        p32 - u2[1] / 3.0 - v02[3] / 3.0 + w2[1] / 9.0,
                        points11[0] - u1[1] / 3.0 + v11[0] / 3.0 + w1[1] / 9.0,
                        points02[0] - u2[0] / 3.0 + v02[0] / 3.0 + w2[0] / 9.0,
                    ],
                },
            ],
        }
    }
}

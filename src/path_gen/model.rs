use super::{
    gen::{CUTTER_RADIUS_DETAIL, CUTTER_RADIUS_ROUGH},
    utils::*,
};
use crate::{
    cnc::block::Block,
    math::{
        geometry::{
            intersection::{Intersection, IntersectionFinder},
            parametric_form::DifferentialParametricForm,
            surfaces::{ShiftedSurface, XZPlane},
        },
        utils::vec_64_to_32,
    },
};
use itertools::Itertools;
use kiddo::KdTree;
use nalgebra::{geometry::Rotation2, point, vector, Point3, Vector2, Vector3};
use std::collections::HashMap;

const PLANE_SIZE: f64 = 7.0;
pub const PLANE_CENTER: Vector3<f64> = vector![0.0, 0.0, 2.5];

const NUMERICAL_STEP: f64 = 0.005;
pub const INTERSECTION_STEP: f64 = 0.01;
const KDTREE_SEARCH_RADIUS: f64 = INTERSECTION_STEP * 5.0;
const SILHOUETTE_GUIDE_POINT: Point3<f64> = point![-2.0, 0.0, 2.5];
const PERTURBATION: f64 = 0.1;
const INTER_COOLDOWN: usize = 15;

pub const BLOCK_SIZE: f32 = 150.0;
pub const BLOCK_HEIGHT: f32 = 50.0;
pub const BLOCK_BASE: f32 = 16.0;

pub const MODEL_SCALE: f32 = 30.0;

const HEIGHTMAP_SAMPLING: usize = 200;
const HEIGHTMAP_PARAMETER_SAMPLING: usize = 325;
const BLOCK_CONVERT: f32 = HEIGHTMAP_SAMPLING as f32 / BLOCK_SIZE;

pub const BODY_ID: usize = 181;
pub const LEFT_SHACKLE_ID: usize = 210;
pub const RIGHT_SHACKLE_ID: usize = 239;
pub const LEFT_SHIELD_ID: usize = 273;
pub const RIGHT_SHIELD_ID: usize = 256;
pub const LEFT_SCREW_ID: usize = 307;
pub const RIGHT_SCREW_ID: usize = 290;

pub const INTERSECTIONS: [InterGuide; 8] = [
    InterGuide {
        id_0: BODY_ID,
        id_1: LEFT_SHIELD_ID,
        guide: point![0.0, 1.0, 3.0],
        shifted_sign_0: 1.0,
        shifted_sign_1: 1.0,
    },
    InterGuide {
        id_0: BODY_ID,
        id_1: RIGHT_SHIELD_ID,
        guide: point![0.0, 1.0, 1.0],
        shifted_sign_0: 1.0,
        shifted_sign_1: 1.0,
    },
    InterGuide {
        id_0: LEFT_SHIELD_ID,
        id_1: LEFT_SCREW_ID,
        guide: point![0.0, 1.0, 3.5],
        shifted_sign_0: 1.0,
        shifted_sign_1: -1.0,
    },
    InterGuide {
        id_0: RIGHT_SHIELD_ID,
        id_1: RIGHT_SCREW_ID,
        guide: point![0.0, 1.0, 1.5],
        shifted_sign_0: 1.0,
        shifted_sign_1: -1.0,
    },
    InterGuide {
        id_0: BODY_ID,
        id_1: LEFT_SHACKLE_ID,
        guide: point![-1.0, 1.0, 4.0],
        shifted_sign_0: 1.0,
        shifted_sign_1: 1.0,
    },
    InterGuide {
        id_0: BODY_ID,
        id_1: LEFT_SHACKLE_ID,
        guide: point![-1.0, 1.0, 3.0],
        shifted_sign_0: 1.0,
        shifted_sign_1: 1.0,
    },
    InterGuide {
        id_0: BODY_ID,
        id_1: RIGHT_SHACKLE_ID,
        guide: point![-1.0, 1.0, 2.0],
        shifted_sign_0: 1.0,
        shifted_sign_1: 1.0,
    },
    InterGuide {
        id_0: BODY_ID,
        id_1: RIGHT_SHACKLE_ID,
        guide: point![-1.0, 1.0, 1.0],
        shifted_sign_0: 1.0,
        shifted_sign_1: 1.0,
    },
];

pub const LEFT_SHACKLE_INTERS: [usize; 2] = [4, 5];
pub const RIGHT_SHACKLE_INTERS: [usize; 2] = [6, 7];
pub const LEFT_SCREW_INTER: usize = 2;
pub const RIGHT_SCREW_INTER: usize = 3;
pub const LEFT_SHIELD_INTER: usize = 0;
pub const RIGHT_SHIELD_INTER: usize = 1;

const HOLE_INTERSECTIONS: [InterPlaneGuide; 2] = [
    InterPlaneGuide {
        id: LEFT_SHACKLE_ID,
        guide: point![-1.25, 0.0, 3.5],
    },
    InterPlaneGuide {
        id: RIGHT_SHACKLE_ID,
        guide: point![-1.25, 0.0, 1.5],
    },
];

pub struct Model {
    pub surfaces: HashMap<usize, Box<dyn DifferentialParametricForm<2, 3> + Send + Sync>>,
}

impl Model {
    pub fn new(
        surfaces: Vec<Box<dyn DifferentialParametricForm<2, 3> + Send + Sync>>,
        ids: Vec<usize>,
    ) -> Self {
        Self {
            surfaces: HashMap::from_iter(ids.into_iter().zip(surfaces)),
        }
    }

    pub fn sampled_block(&self) -> Block {
        let mut block = Block::new(
            vector![HEIGHTMAP_SAMPLING, HEIGHTMAP_SAMPLING],
            vector![BLOCK_SIZE, BLOCK_SIZE, BLOCK_HEIGHT],
        );

        let sampling = *block.sampling();
        for x in 0..sampling.x {
            for y in 0..sampling.y {
                *block.height_mut(x, y) = BLOCK_BASE;
            }
        }

        for (id, surface) in &self.surfaces {
            let multiplier = if *id == LEFT_SCREW_ID || *id == RIGHT_SCREW_ID {
                -1.0
            } else {
                1.0
            };

            let shifted = ShiftedSurface::new(
                surface.as_ref(),
                multiplier * (CUTTER_RADIUS_ROUGH / MODEL_SCALE) as f64,
            );

            Self::create_height(&shifted, 0.0, &mut block);
            Self::create_height(surface.as_ref(), CUTTER_RADIUS_ROUGH, &mut block);
        }

        block
    }

    fn create_height(surface: &dyn DifferentialParametricForm<2, 3>, bump: f32, block: &mut Block) {
        let bounds = surface.bounds();
        let u_step = (bounds.x.1 - bounds.x.0) / HEIGHTMAP_PARAMETER_SAMPLING as f64;
        let v_step = (bounds.y.1 - bounds.y.0) / HEIGHTMAP_PARAMETER_SAMPLING as f64;

        // Intentionally skip the last sample so that dealing with numerical errors of `u` and
        // `v` at the border is not necessary
        let mut u = bounds.x.0;
        for _ in 0..HEIGHTMAP_PARAMETER_SAMPLING {
            let mut v = bounds.y.0;
            for _ in 0..HEIGHTMAP_PARAMETER_SAMPLING {
                let mut value =
                    vec_64_to_32(surface.value(&vector![u, v]).coords - PLANE_CENTER) * MODEL_SCALE;

                value.y += BLOCK_BASE + bump;

                let x = ((value.x as f32 + BLOCK_SIZE * 0.5) * BLOCK_CONVERT).floor() as i64;
                let y = ((value.z as f32 + BLOCK_SIZE * 0.5) * BLOCK_CONVERT).floor() as i64;

                if x >= 0
                    && y >= 0
                    && x < block.sampling().x as i64
                    && y < block.sampling().y as i64
                    && block.height(x as usize, y as usize) < value.y as f32 - CUTTER_RADIUS_ROUGH
                {
                    *block.height_mut(x as usize, y as usize) =
                        value.y as f32 - CUTTER_RADIUS_ROUGH;
                }

                v += v_step;
            }

            u += u_step;
        }
    }

    pub fn silhouette(&self) -> Option<Intersection> {
        let plane = Self::plane();

        let intersections = [BODY_ID, LEFT_SHACKLE_ID, RIGHT_SHACKLE_ID]
            .map(|id| &self.surfaces[&id])
            .iter()
            .filter_map(|s| {
                let mut finder = IntersectionFinder::new(&plane, s.as_ref());
                finder.numerical_step = NUMERICAL_STEP;
                finder.intersection_step = INTERSECTION_STEP;
                finder.guide_point = Some(SILHOUETTE_GUIDE_POINT);
                finder.find()
            })
            .collect_vec();

        intersections
            .into_iter()
            .reduce(|x, y| looped_outer_intersection_sum(x, y, false, false))
    }

    pub fn elevated_silhouette(&self) -> Option<Intersection> {
        let dist = (CUTTER_RADIUS_DETAIL / MODEL_SCALE) as f64;
        let mut plane = Self::plane();
        plane.height(dist);

        let intersections = [BODY_ID, LEFT_SHACKLE_ID, RIGHT_SHACKLE_ID]
            .map(|id| &self.surfaces[&id])
            .iter()
            .map(|s| {
                let shifted = ShiftedSurface::new(s.as_ref(), dist);
                let mut finder = IntersectionFinder::new(&plane, &shifted);
                finder.numerical_step = NUMERICAL_STEP;
                finder.intersection_step = INTERSECTION_STEP;
                finder.guide_point = Some(SILHOUETTE_GUIDE_POINT);
                let mut intersection = finder.find().unwrap();
                intersection
                    .points
                    .iter_mut()
                    .for_each(|p| p.point.y = dist);
                intersection
            })
            .collect_vec();

        intersections
            .into_iter()
            .reduce(|x, y| looped_outer_intersection_sum(x, y, true, false))
    }

    pub fn find_model_intersections(&self) -> [Intersection; INTERSECTIONS.len()] {
        INTERSECTIONS.map(|ig| {
            let shifted_0 = ShiftedSurface::new(
                self.surfaces[&ig.id_0].as_ref(),
                ig.shifted_sign_0 * (CUTTER_RADIUS_DETAIL / MODEL_SCALE) as f64,
            );
            let shifted_1 = ShiftedSurface::new(
                self.surfaces[&ig.id_1].as_ref(),
                ig.shifted_sign_1 * (CUTTER_RADIUS_DETAIL / MODEL_SCALE) as f64,
            );

            let mut finder = IntersectionFinder::new(&shifted_0, &shifted_1);
            finder.numerical_step = NUMERICAL_STEP;
            finder.intersection_step = INTERSECTION_STEP;
            finder.guide_point = Some(ig.guide);
            let err = format!(
                "Intersection between {} and {} not found!",
                ig.id_0, ig.id_1
            );
            finder.find().expect(&err)
        })
    }

    pub fn find_holes(&self) -> [Intersection; HOLE_INTERSECTIONS.len()] {
        let dist = (CUTTER_RADIUS_DETAIL / MODEL_SCALE) as f64;
        let mut plane = Self::plane();
        plane.height(dist);

        let shifted_body = ShiftedSurface::new(self.surfaces[&BODY_ID].as_ref(), dist);

        let mut finder = IntersectionFinder::new(&plane, &shifted_body);
        finder.numerical_step = NUMERICAL_STEP;
        finder.intersection_step = INTERSECTION_STEP;
        finder.guide_point = Some(SILHOUETTE_GUIDE_POINT);
        let mut body_inter = finder
            .find()
            .expect("Could not find intersection of the main body with the plane");
        body_inter.reverse();

        HOLE_INTERSECTIONS.map(|ig| {
            let shifted = ShiftedSurface::new(self.surfaces[&ig.id].as_ref(), dist);
            let mut finder = IntersectionFinder::new(&plane, &shifted);
            finder.numerical_step = NUMERICAL_STEP;
            finder.intersection_step = INTERSECTION_STEP;
            finder.guide_point = Some(ig.guide);
            let err = format!("Intersection between {} and the plane not found!", ig.id);
            let inter = finder.find().expect(&err);
            looped_outer_intersection_sum(inter, body_inter.clone(), true, true)
        })
    }

    pub fn plane() -> XZPlane {
        XZPlane::new(
            (PLANE_CENTER - vector![PLANE_SIZE / 2.0, 0.0, PLANE_SIZE / 2.0]).into(),
            vector![PLANE_SIZE, PLANE_SIZE],
        )
    }
}

/// intersections have to be calculated with XZPlane as surface_0
fn looped_outer_intersection_sum(
    inter_current: Intersection,
    inter_second: Intersection,
    start_in_the_middle: bool,
    constant_direction: bool,
) -> Intersection {
    const MAX_POINTS: usize = 3000;

    //     return Intersection {
    //         looped: true,
    //         points: inter_current
    //             .points
    //             .iter()
    //             .chain(inter_second.points.iter())
    //             .copied()
    //             .collect_vec(),
    //     };

    // To avoid KdTree lumping all points on one axis
    let perturbation = Rotation2::new(PERTURBATION);
    // Assume all indexing is correct
    let mut inter_current = &inter_current;
    let mut inter_second = &inter_second;
    let mut kdtree_current = intersection_kdtree(inter_current);
    let mut kdtree_current = &mut kdtree_current;
    let mut kdtree_second = intersection_kdtree(inter_second);
    let mut kdtree_second = &mut kdtree_second;

    let mut sum_points =
        Vec::with_capacity(inter_current.points.capacity() + inter_second.points.capacity());

    let mut current_idx = if start_in_the_middle {
        inter_current.points.len() as i64 / 2
    } else {
        0
    } + 2;
    sum_points.push(inter_current.points[current_idx as usize - 2]);
    sum_points.push(inter_current.points[current_idx as usize - 1]);

    let mut idx_step = 1;
    let mut found_intersection = false;
    let mut last_found = INTER_COOLDOWN;

    // Assume that the silhouette has no holes
    while sum_points.first() != sum_points.last() {
        if sum_points.len() == inter_current.points.len() && !found_intersection {
            // No points are close enough to the second curve
            break;
        }

        let len = sum_points.len();
        let direction = sum_points[len - 1].surface_0 - sum_points[len - 2].surface_0;
        let normal = vector![-direction.y, direction.x];
        let cur_point = perturbation * sum_points[len - 1].surface_0;
        let neighbour = kdtree_second.nearest_one(
            &[cur_point.x, cur_point.y],
            &(|p_0, p_1| (p_0[0] - p_1[0]).abs() + (p_0[1] - p_1[1]).abs()),
        );

        if neighbour.0 <= KDTREE_SEARCH_RADIUS && last_found >= INTER_COOLDOWN {
            last_found = 0;
            found_intersection = true;
            // Assume the neighbour creates an intersection
            let neigh_dir = inter_second.points[neighbour.1].surface_0
                - inter_second.points[(neighbour.1 as i64 - 1)
                    .rem_euclid(inter_second.points.len() as i64)
                    as usize]
                    .surface_0;

            idx_step = if constant_direction || Vector2::dot(&neigh_dir, &normal) < 0.0 {
                1
            } else {
                -1
            };

            std::mem::swap(&mut inter_current, &mut inter_second);
            std::mem::swap(&mut kdtree_current, &mut kdtree_second);
            current_idx = neighbour.1 as i64;
        } else {
            last_found += 1;
        }

        sum_points.push(inter_current.points[current_idx as usize]);
        current_idx += idx_step;
        current_idx = current_idx.rem_euclid(inter_current.points.len() as i64);

        if sum_points.len() > MAX_POINTS {
            break;
        }
    }

    Intersection {
        looped: true,
        points: sum_points,
    }
}

fn intersection_kdtree(intersection: &Intersection) -> KdTree<f64, 2> {
    let mut kdtree = KdTree::new();
    let rot = Rotation2::new(PERTURBATION);

    for (idx, point) in intersection.points.iter().enumerate() {
        let point_perturbed = rot * point.surface_0;
        kdtree.add(&[point_perturbed.x, point_perturbed.y], idx);
    }

    kdtree
}

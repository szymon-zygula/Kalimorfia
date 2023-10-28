use crate::{
    cnc::block::Block,
    math::{
        geometry::{
            intersection::{Intersection, IntersectionFinder},
            parametric_form::DifferentialParametricForm,
            surfaces::XZPlane,
        },
        utils::vec_64_to_32,
    },
};
use itertools::Itertools;
use kiddo::KdTree;
use nalgebra::{geometry::Rotation2, vector, Point3, Vector2, Vector3};

const PLANE_SIZE: f64 = 7.0;
const PLANE_CENTER: [f64; 3] = [0.0, 0.0, 2.5];
const NUMERICAL_STEP: f64 = 0.005;
const INTERSECTION_STEP: f64 = 0.01;
const KDTREE_SEARCH_RADIUS: f64 = INTERSECTION_STEP * 5.0;
const GUIDE_POINT: [f64; 3] = [-2.0, 0.0, 2.5];
const INTERSECTION_SUM_START_POINT: usize = 0;
const PERTURBATION: f64 = 0.1;
const INTER_COOLDOWN: usize = 15;
const BLOCK_SIZE: f32 = 150.0;
const BLOCK_HEIGHT: f32 = 50.0;
const MODEL_SCALE: f32 = 30.0;
const HEIGHTMAP_SAMPLING: usize = 250;
const HEIGHTMAP_PARAMETER_SAMPLING: usize = 300;
const BLOCK_BASE: f32 = 16.0;

pub struct Model {
    surfaces: Vec<Box<dyn DifferentialParametricForm<2, 3>>>,
}

impl Model {
    pub fn new(surfaces: Vec<Box<dyn DifferentialParametricForm<2, 3>>>) -> Self {
        Self { surfaces }
    }

    pub fn sampled_block(&self) -> Block {
        let origin = Vector3::from_row_slice(&PLANE_CENTER);
        let block_convert = HEIGHTMAP_SAMPLING as f32 / BLOCK_SIZE;
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

        for surface in &self.surfaces {
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
                        vec_64_to_32(surface.value(&vector![u, v]).coords - origin) * MODEL_SCALE;

                    value.y += BLOCK_BASE;
                    let x = ((value.x as f32 + BLOCK_SIZE * 0.5) * block_convert).round() as usize;
                    let y = ((value.z as f32 + BLOCK_SIZE * 0.5) * block_convert).round() as usize;

                    if block.height(x, y) < value.y as f32 {
                        *block.height_mut(x, y) = value.y as f32;
                    }

                    v += v_step;
                }

                u += u_step;
            }
        }

        block
    }

    pub fn silhouette(&self) -> Option<Intersection> {
        let plane = XZPlane::new(
            Point3::from_slice(&PLANE_CENTER) - vector![PLANE_SIZE / 2.0, 0.0, PLANE_SIZE / 2.0],
            vector![PLANE_SIZE, PLANE_SIZE],
        );

        let mut intersections = self
            .surfaces
            .iter()
            .filter_map(|s| {
                let mut finder = IntersectionFinder::new(&plane, s.as_ref());
                finder.numerical_step = NUMERICAL_STEP;
                finder.intersection_step = INTERSECTION_STEP;
                finder.guide_point = Some(Point3::from_slice(&GUIDE_POINT));
                finder.find()
            })
            .collect_vec();

        // Sort to start with the body of the padlock
        intersections.sort_by(|a, b| b.points.len().cmp(&a.points.len()));

        intersections
            .into_iter()
            .reduce(looped_outer_intersection_sum)
    }
}

fn looped_outer_intersection_sum(
    inter_current: Intersection,
    inter_second: Intersection,
) -> Intersection {
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
    sum_points.push(inter_current.points[0]);
    sum_points.push(inter_current.points[1]);
    let mut current_idx = INTERSECTION_SUM_START_POINT as i64 + 2;
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
                - inter_second.points[neighbour.1 - 1].surface_0;

            idx_step = if Vector2::dot(&neigh_dir, &normal) < 0.0 {
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
    }

    Intersection {
        looped: true,
        points: sum_points,
    }
}

fn intersection_kdtree(intersection: &Intersection) -> KdTree<f64, 2> {
    let mut kdtree = KdTree::new();

    for (idx, point) in intersection.points.iter().enumerate() {
        let point_perturbed = Rotation2::new(PERTURBATION) * point.surface_0;
        kdtree.add(&[point_perturbed.x, point_perturbed.y], idx);
    }

    kdtree
}

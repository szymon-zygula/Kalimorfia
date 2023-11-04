use super::model::{Model, BLOCK_SIZE, INTERSECTION_STEP, MODEL_SCALE, PLANE_CENTER};
use crate::{
    cnc::{
        block::Block,
        mill::{Cutter, CutterShape},
        program as cncp,
    },
    math::{
        geometry::intersection::{Intersection, IntersectionPoint},
        utils::vec_64_to_32,
    },
};
use itertools::Itertools;
use nalgebra::{vector, Point2, Vector2, Vector3};
use ordered_float::NotNan;
use rayon::prelude::*;
use std::collections::{BTreeMap, HashMap};

const SAFE_CONTOUR_ADD: usize = 3;
const INTERSECTION_IN_BLOCK: f32 = INTERSECTION_STEP as f32 * MODEL_SCALE;

const SAFE_HEIGHT: f32 = 66.0;
const CUTTER_DIAMETER_ROUGH: f32 = 16.0;
const CUTTER_RADIUS_ROUGH: f32 = CUTTER_DIAMETER_ROUGH * 0.5;
const CUTTER_RADIUS_ROUGH_SQRT_2: f32 = CUTTER_RADIUS_ROUGH * std::f32::consts::SQRT_2 * 0.5;
const BASE_HEIGHT: f32 = 16.0;

const CUTTER_DIAMETER_FLAT: f32 = 10.0;
const CUTTER_RADIUS_FLAT: f32 = 0.5 * CUTTER_DIAMETER_FLAT;
const CUTTER_HEIGHT_FLAT: f32 = 4.0 * CUTTER_DIAMETER_FLAT;
const FLAT_EPS: f32 = 0.1 * CUTTER_RADIUS_FLAT;

const CUTTER_DIAMETER_DETAIL: f32 = 8.0;
pub const CUTTER_RADIUS_DETAIL: f32 = 0.5 * CUTTER_DIAMETER_DETAIL;
const HOLE_NET_SAFE_DIST: f32 = CUTTER_RADIUS_DETAIL * 0.3;

pub fn rough(model: &Model) -> cncp::Program {
    const UPPER_PLANE_HEIGHT: f32 = 35.0;
    const LOWER_PLANE_HEIGHT: f32 = 20.0;
    const CUTTER_HEIGHT: f32 = 4.0 * CUTTER_DIAMETER_ROUGH;
    const SPACING: f32 = CUTTER_DIAMETER_ROUGH * 0.5;
    const SAMPLING: f32 = 1.0;

    let heightmap = model.sampled_block();
    let mut locs = initial_locations();
    locs.push(vector![
        BLOCK_SIZE * 0.5 + SPACING,
        BLOCK_SIZE * 0.5 + SPACING * 2.0,
        SAFE_HEIGHT
    ]);

    locs.extend(rough_plane(
        UPPER_PLANE_HEIGHT,
        &heightmap,
        SPACING,
        SAMPLING,
    ));

    let mut lower_plane = rough_plane(LOWER_PLANE_HEIGHT, &heightmap, SPACING, SAMPLING);
    lower_plane.reverse();
    locs.extend(lower_plane);

    add_ending_locs(&mut locs);

    cncp::Program::from_locations(
        locs,
        Cutter {
            height: CUTTER_HEIGHT,
            diameter: CUTTER_DIAMETER_ROUGH,
            shape: CutterShape::Ball,
        },
    )
}

fn rough_plane(height: f32, heightmap: &Block, spacing: f32, sampling: f32) -> Vec<Vector3<f32>> {
    (0..(BLOCK_SIZE / spacing + 4.0) as usize)
        .into_par_iter()
        .flat_map(|i| {
            let x = 0.5 * BLOCK_SIZE + spacing - spacing * i as f32;
            let mut line = rough_line(height, x, heightmap, spacing, sampling);
            if i % 2 == 1 {
                line.reverse();
            }

            line
        })
        .collect()
}

fn get_height(bx: f32, by: f32, hsamx: f32, hsamy: f32, block: &Block) -> Option<f32> {
    (bx >= 0.0 && by >= 0.0 && bx < hsamx && by < hsamy)
        .then(|| block.height(bx as usize, by as usize))
}

fn rough_max(bx: f32, by: f32, hsamx: f32, hsamy: f32, block: &Block) -> f32 {
    [
        (bx, by),
        (bx + CUTTER_RADIUS_ROUGH, by),
        (bx, by + CUTTER_RADIUS_ROUGH),
        (bx - CUTTER_RADIUS_ROUGH, by),
        (bx, by - CUTTER_RADIUS_ROUGH),
        (
            bx + CUTTER_RADIUS_ROUGH_SQRT_2,
            by + CUTTER_RADIUS_ROUGH_SQRT_2,
        ),
        (
            bx + CUTTER_RADIUS_ROUGH_SQRT_2,
            by - CUTTER_RADIUS_ROUGH_SQRT_2,
        ),
        (
            bx - CUTTER_RADIUS_ROUGH_SQRT_2,
            by + CUTTER_RADIUS_ROUGH_SQRT_2,
        ),
        (
            bx - CUTTER_RADIUS_ROUGH_SQRT_2,
            by - CUTTER_RADIUS_ROUGH_SQRT_2,
        ),
    ]
    .into_iter()
    .filter_map(|(x, y)| get_height(x, y, hsamx, hsamy, block))
    .fold(0.0, f32::max)
}

fn rough_line(
    height: f32,
    x: f32,
    heightmap: &Block,
    spacing: f32,
    sampling: f32,
) -> Vec<Vector3<f32>> {
    let mut y = 0.5 * BLOCK_SIZE + spacing * 2.0;
    let width = BLOCK_SIZE + 4.0 * spacing;
    let samples = (width / sampling) as usize + 1;
    let mut locs: Vec<Vector3<f32>> = Vec::new();
    let ss = heightmap.sample_size();
    let hsam = heightmap.sampling();
    let hsamx = hsam.x as f32;
    let hsamy = hsam.y as f32;

    for _ in 0..samples {
        let bx = ((x + BLOCK_SIZE * 0.5) / ss.x).floor();
        let by = ((y + BLOCK_SIZE * 0.5) / ss.y).floor();

        let z = if bx >= 0.0 && by >= 0.0 && bx < hsamx && by < hsamy {
            f32::max(rough_max(bx, by, hsamx, hsamy, heightmap), height)
        } else {
            height
        };

        let new = vector![x, y, z];
        let len = locs.len();

        if len >= 2 && locs[len - 1].z == z && locs[len - 2].z == z {
            locs[len - 1] = new;
        } else {
            locs.push(new);
        }

        y -= sampling;
    }

    locs
}

pub fn flat(model: &Model) -> Option<cncp::Program> {
    let mut locs = initial_locations();
    locs.extend_from_slice(&[
        vector![
            -BLOCK_SIZE * 0.5 - CUTTER_DIAMETER_FLAT,
            BLOCK_SIZE * 0.5 + CUTTER_DIAMETER_FLAT,
            SAFE_HEIGHT
        ],
        vector![
            -BLOCK_SIZE * 0.5 - CUTTER_DIAMETER_FLAT,
            BLOCK_SIZE * 0.5 + CUTTER_DIAMETER_FLAT,
            BASE_HEIGHT
        ],
    ]);

    let silhouette = model.silhouette()?;

    locs.extend(flat_mow(&silhouette));
    locs.extend(flat_silhouette(&silhouette)?);

    add_ending_locs(&mut locs);

    Some(cncp::Program::from_locations(
        locs,
        Cutter {
            height: CUTTER_HEIGHT_FLAT,
            diameter: CUTTER_DIAMETER_FLAT,
            shape: CutterShape::Cylinder,
        },
    ))
}

fn flat_mow(silhouette: &Intersection) -> Vec<Vector3<f32>> {
    let (bottom, top) = silhouette
        .points
        .iter()
        .map(|p| {
            (
                NotNan::new((p.point.z - PLANE_CENTER[2]) * MODEL_SCALE as f64).unwrap(),
                *p,
            )
        })
        .partition::<BTreeMap<NotNan<f64>, IntersectionPoint>, _>(|(_, p)| {
            p.point.x - PLANE_CENTER[0] > 0.0
        });

    let mut locs = flat_partition_paths(top, -1.0);
    locs.extend(flat_partition_paths(bottom, 1.0).iter().rev());
    locs
}

fn flat_partition_paths(
    border: BTreeMap<NotNan<f64>, IntersectionPoint>,
    approach: f64,
) -> Vec<Vector3<f32>> {
    let mut locs = Vec::new();

    let mut y = (-BLOCK_SIZE * 0.5 - CUTTER_RADIUS_FLAT) as f64;
    while y < (BLOCK_SIZE * 0.5 + CUTTER_RADIUS_FLAT) as f64 {
        flat_partition_path_pair(
            NotNan::new(y).unwrap(),
            NotNan::new(y + (CUTTER_DIAMETER_FLAT - FLAT_EPS) as f64).unwrap(),
            &border,
            &mut locs,
            NotNan::new(approach).unwrap(),
        );

        y += (CUTTER_DIAMETER_FLAT - FLAT_EPS) as f64 * 2.0;
    }

    locs
}

fn flat_partition_path_pair(
    y: NotNan<f64>,
    y_limit: NotNan<f64>,
    border: &BTreeMap<NotNan<f64>, IntersectionPoint>,
    locs: &mut Vec<Vector3<f32>>,
    approach: NotNan<f64>,
) {
    const LIMIT_ACCURACY: usize = 10;
    // Do not touch the model while mowing the grass
    const CUTTER_SAFE_DISTANCE_MULTIPLIER: f32 = 1.1;

    let x_start = *approach as f32 * (0.5 * BLOCK_SIZE + CUTTER_DIAMETER_FLAT);

    locs.push(vector![x_start, *y as f32, BASE_HEIGHT]);

    for i in 0..LIMIT_ACCURACY {
        let t = i as f64 / (LIMIT_ACCURACY as f64 - 1.0);
        let y_interpol = y * (1.0 - t) + (y_limit) * t;

        let x_limit = border
            .range(
                (y_interpol - CUTTER_RADIUS_FLAT as f64)..(y_interpol + CUTTER_RADIUS_FLAT as f64),
            )
            .map(|(_, p)| {
                approach.as_f32()
                    * NotNan::new((p.point.x - PLANE_CENTER[0]) as f32 * MODEL_SCALE).unwrap()
            })
            .max()
            .map(|p| approach.as_f32() * p)
            .unwrap_or(-NotNan::new(5.0).unwrap() * approach.as_f32())
            + *approach as f32 * CUTTER_RADIUS_FLAT * CUTTER_SAFE_DISTANCE_MULTIPLIER;

        locs.push(vector![*x_limit, *y_interpol as f32, BASE_HEIGHT]);
    }

    locs.push(vector![x_start, *y_limit as f32, BASE_HEIGHT]);
}

fn flat_silhouette(silhouette: &Intersection) -> Option<Vec<Vector3<f32>>> {
    let len = silhouette.points.len();
    let mut locs = silhouette
        .points
        .iter()
        .map(|p| p.point.xz())
        .cycle()
        .skip(len / 2) // Model-specific things -- start from the other side
        .take(len + SAFE_CONTOUR_ADD) // make sure that the whole silhouette is cut with cutter moving
        .tuple_windows()
        .filter_map(|(a, b)| cutter_at_inter_base::<false>(CUTTER_RADIUS_FLAT, a, b))
        .collect();
    clean_cutter_at_inter_base(&mut locs);

    Some(locs)
}

fn btree_closest(
    btree: &BTreeMap<NotNan<f64>, IntersectionPoint>,
    query: NotNan<f64>,
) -> IntersectionPoint {
    let lower_bound = btree.range(..query).next_back();
    let upper_bound = btree.range(query..).next();

    match (lower_bound, upper_bound) {
        (None, None) => panic!("Empty tree"),
        (None, Some(x)) => *x.1,
        (Some(x), None) => *x.1,
        (Some(x), Some(y)) => {
            if (*x.0 - query).abs() > (*y.0 - query).abs() {
                *x.1
            } else {
                *y.1
            }
        }
    }
}

pub fn detail(model: &Model) -> cncp::Program {
    const CUTTER_DIAMETER: f32 = 8.0;
    const CUTTER_HEIGHT: f32 = 4.0 * CUTTER_DIAMETER;

    let mut locs = initial_locations();
    locs.extend(grill(model));
    locs.extend(inters(model));
    add_ending_locs(&mut locs);

    cncp::Program::from_locations(
        locs,
        Cutter {
            height: CUTTER_HEIGHT,
            diameter: CUTTER_DIAMETER,
            shape: CutterShape::Ball,
        },
    )
}

fn grill(model: &Model) -> Vec<Vector3<f32>> {
    let mut locs = Vec::new();
    let holes = model.find_holes();

    for hole in holes {
        let p0 = hole.points[0].point.xz();
        let p1 = hole.points[1].point.xz();
        let mut first_high = cutter_at_inter_base::<true>(CUTTER_RADIUS_DETAIL, p0, p1).unwrap();
        first_high.z = SAFE_HEIGHT;
        locs.push(first_high);

        let contour = grill_contour(&hole);

        locs.extend(grill_net(&contour));
        locs.extend(
            grill_net(&contour.iter().map(|p| p.yxz()).collect_vec())
                .iter()
                .map(|p| p.yxz()),
        );
        locs.extend(contour);

        let mut last_high = *locs.last().unwrap();
        last_high.z = SAFE_HEIGHT;
        locs.push(last_high);
    }

    locs
}

fn grill_contour(hole: &Intersection) -> Vec<Vector3<f32>> {
    let len = hole.points.len();
    let mut points = hole
        .points
        .iter()
        .map(|p| p.point.xz())
        .cycle()
        .take(len + SAFE_CONTOUR_ADD) // + 3 to make sure that the whole hole
        .tuple_windows()
        .filter_map(|(a, b)| cutter_at_inter_base::<true>(CUTTER_RADIUS_DETAIL, a, b))
        .collect();

    clean_cutter_at_inter_base(&mut points);
    points
}

fn grill_net(contour: &[Vector3<f32>]) -> Vec<Vector3<f32>> {
    let mut locs = Vec::new();

    let x_map: BTreeMap<_, _> = contour
        .iter()
        .map(|p| (NotNan::new(p.x).unwrap(), p.y))
        .collect();

    let (&min_x, _) = x_map.first_key_value().unwrap();
    let (&max_x, _) = x_map.last_key_value().unwrap();
    let span = (min_x - max_x).abs();
    let paths = (2.0 * span / CUTTER_RADIUS_DETAIL).ceil() as i32;
    let x_step = span / paths as f32;

    let mut x = min_x + HOLE_NET_SAFE_DIST;
    for i in 0..paths {
        let max_y = x_map
            .range(x - INTERSECTION_IN_BLOCK..x + INTERSECTION_IN_BLOCK)
            .map(|(_, v)| *v)
            .fold(-f32::INFINITY, f32::max);

        let min_y = x_map
            .range(x - INTERSECTION_IN_BLOCK..x + INTERSECTION_IN_BLOCK)
            .map(|(_, v)| *v)
            .fold(f32::INFINITY, f32::min);

        if i % 2 == 0 {
            locs.push(vector![*x, min_y + HOLE_NET_SAFE_DIST, BASE_HEIGHT]);
            locs.push(vector![*x, max_y - HOLE_NET_SAFE_DIST, BASE_HEIGHT]);
        } else {
            locs.push(vector![*x, max_y - HOLE_NET_SAFE_DIST, BASE_HEIGHT]);
            locs.push(vector![*x, min_y + HOLE_NET_SAFE_DIST, BASE_HEIGHT]);
        }

        x += x_step;
    }

    locs
}

fn inters(model: &Model) -> Vec<Vector3<f32>> {
    let mut locs = Vec::new();
    let intersections = model.find_model_intersections();

    for intersection in intersections {
        let mut initial_locs = intersection
            .points
            .iter()
            .map(|p| {
                (vec_64_to_32(p.point.coords) - vec_64_to_32(PLANE_CENTER)).xzy() * MODEL_SCALE
                    + vector![0.0, 0.0, BASE_HEIGHT - CUTTER_RADIUS_DETAIL]
            })
            .collect_vec();

        if let Some(first_under) = initial_locs.iter().position(|p| p.z <= BASE_HEIGHT) {
            let first_over = initial_locs
                .iter()
                .skip(first_under + 1)
                .position(|p| p.z > BASE_HEIGHT)
                .unwrap()
                + first_under
                + 1;

            initial_locs.drain(first_under..first_over);
            let mut new_start = initial_locs.split_off(first_under);
            new_start.extend(initial_locs);
            initial_locs = new_start;
        } else {
            for i in 0..SAFE_CONTOUR_ADD {
                initial_locs.push(initial_locs[i]);
            }
        }

        let mut first_safe = *initial_locs.first().unwrap();
        first_safe.z = SAFE_HEIGHT;
        locs.push(first_safe);

        locs.extend(initial_locs);

        let mut last_safe = *locs.last().unwrap();
        last_safe.z = SAFE_HEIGHT;
        locs.push(last_safe);
    }

    locs
}

fn cutter_at_inter_base<const INV_NORM: bool>(
    radius: f32,
    a: Point2<f64>,
    b: Point2<f64>,
) -> Option<Vector3<f32>> {
    if a == b {
        return None;
    }

    let center = vector![
        ((a.x + b.x) * 0.5 - PLANE_CENTER[0]) as f32 * MODEL_SCALE,
        ((a.y + b.y) * 0.5 - PLANE_CENTER[2]) as f32 * MODEL_SCALE,
        BASE_HEIGHT
    ];
    let mut normal = vector![(-a.y + b.y) as f32, (a.x - b.x) as f32, 0.0].normalize() * radius;

    if INV_NORM {
        normal = -normal;
    }

    Some(center + normal)
}

fn clean_cutter_at_inter_base(vec: &mut Vec<Vector3<f32>>) {
    let mut hashmap = HashMap::new();
    let mut cut_ranges = Vec::new();

    #[allow(clippy::needless_range_loop)]
    for i in 0..vec.len() - SAFE_CONTOUR_ADD {
        let cur_round = round_vec(&vec[i]);
        let prev = hashmap.get(&cur_round);

        if let Some(&previous) = prev {
            // Assume that 0 is always a correct point to
            if i - previous < 4 || previous == 0 {
                *hashmap.get_mut(&cur_round).unwrap() = i;
            } else {
                cut_ranges.push(previous..i);
            }
        } else {
            hashmap.insert(cur_round, i);
        }
    }

    let mut i = 0;
    let mut j = 0;
    while j < vec.len() {
        if cut_ranges.iter().any(|r| r.contains(&j)) {
            j += 1;
            continue;
        }

        vec[i] = vec[j];
        i += 1;
        j += 1;
    }

    vec.resize(i, vector![0.0, 0.0, 0.0]);
}

fn round_vec(vec: &Vector3<f32>) -> Vector2<i32> {
    const ROUND_POWER: f32 = 0.03 / INTERSECTION_STEP as f32;
    vector![
        (vec.x * ROUND_POWER).round() as i32,
        (vec.y * ROUND_POWER).round() as i32
    ]
}

pub fn signa(model: &Model) -> cncp::Program {
    const CUTTER_DIAMETER: f32 = 1.0;
    const CUTTER_HEIGHT: f32 = 4.0 * CUTTER_DIAMETER;

    let mut locs = initial_locations();
    add_ending_locs(&mut locs);

    cncp::Program::from_locations(
        locs,
        Cutter {
            height: CUTTER_HEIGHT,
            diameter: CUTTER_DIAMETER,
            shape: CutterShape::Ball,
        },
    )
}

fn initial_locations() -> Vec<Vector3<f32>> {
    vec![vector![0.0, 0.0, 66.0]]
}

fn add_ending_locs(locs: &mut Vec<Vector3<f32>>) {
    let mut safe = *locs.last().unwrap();
    safe.z = SAFE_HEIGHT;
    locs.push(safe);
    locs.push(vector![0.0, 0.0, SAFE_HEIGHT]);
}

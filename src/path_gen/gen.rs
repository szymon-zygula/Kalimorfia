use super::model::{Model, BLOCK_SIZE, MODEL_SCALE, PLANE_CENTER};
use crate::{
    cnc::{
        block::Block,
        mill::{Cutter, CutterShape},
        program as cncp,
    },
    math::geometry::intersection::{Intersection, IntersectionPoint},
};
use itertools::Itertools;
use nalgebra::{vector, Vector3};
use ordered_float::NotNan;
use rayon::prelude::*;
use std::collections::BTreeMap;

const SAFE_HEIGHT: f32 = 66.0;
const CUTTER_DIAMETER_ROUGH: f32 = 16.0;
const CUTTER_RADIUS_ROUGH: f32 = CUTTER_DIAMETER_ROUGH * 0.5;
const CUTTER_RADIUS_ROUGH_SQRT_2: f32 = CUTTER_RADIUS_ROUGH * std::f32::consts::SQRT_2 * 0.5;
const BASE_HEIGHT: f32 = 16.0;

const CUTTER_DIAMETER_FLAT: f32 = 10.0;
const CUTTER_RADIUS_FLAT: f32 = 0.5 * CUTTER_DIAMETER_FLAT;
const CUTTER_HEIGHT_FLAT: f32 = 4.0 * CUTTER_DIAMETER_FLAT;
const FLAT_EPS: f32 = 0.1 * CUTTER_RADIUS_FLAT;

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
            .unwrap_or(NotNan::new(0.0).unwrap())
            + *approach as f32 * CUTTER_RADIUS_FLAT * CUTTER_SAFE_DISTANCE_MULTIPLIER;

        locs.push(vector![*x_limit, *y_interpol as f32, BASE_HEIGHT]);
    }

    locs.push(vector![x_start, *y_limit as f32, BASE_HEIGHT]);
}

fn flat_silhouette(silhouette: &Intersection) -> Option<Vec<Vector3<f32>>> {
    let len = silhouette.points.len();

    Some(
        silhouette
            .points
            .iter()
            .map(|p| p.point.xz())
            .cycle()
            .skip(len / 2) // Model-specific things -- start from the other side
            .take(len + 3) // + 3 to make sure that the whole silhouette is cut with cutter moving
            .tuple_windows()
            .filter_map(|(a, b)| {
                if a == b {
                    return None;
                }

                let center = vector![
                    ((a.x + b.x) * 0.5 - PLANE_CENTER[0]) as f32 * MODEL_SCALE,
                    ((a.y + b.y) * 0.5 - PLANE_CENTER[2]) as f32 * MODEL_SCALE,
                    BASE_HEIGHT
                ];
                let normal = vector![(-a.y + b.y) as f32, (a.x - b.x) as f32, 0.0].normalize()
                    * CUTTER_RADIUS_FLAT;
                Some(center + normal)
            })
            .collect(),
    )
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

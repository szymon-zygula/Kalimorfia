use super::{model::*, svg};
use crate::{
    cnc::{
        block::Block,
        mill::{Cutter, CutterShape},
        program as cncp,
    },
    math::{
        geometry::{
            intersection::{Intersection, IntersectionPoint},
            parametric_form::DifferentialParametricForm,
            surfaces::ShiftedSurface,
        },
        utils::vec_64_to_32,
    },
};
use itertools::Itertools;
use nalgebra::{vector, Point2, Vector2, Vector3};
use ordered_float::NotNan;
use rayon::prelude::*;
use std::{
    collections::{BTreeMap, HashMap},
    mem::MaybeUninit,
};

const SAFE_CONTOUR_ADD: usize = 3;
const INTERSECTION_IN_BLOCK: f32 = INTERSECTION_STEP as f32 * MODEL_SCALE;

pub const SAFE_HEIGHT: f32 = 66.0;
const CUTTER_DIAMETER_ROUGH: f32 = 16.0;
pub const CUTTER_RADIUS_ROUGH: f32 = CUTTER_DIAMETER_ROUGH * 0.5;
const CUTTER_RADIUS_ROUGH_SQRT_2: f32 = CUTTER_RADIUS_ROUGH * std::f32::consts::SQRT_2 * 0.5;
const BASE_HEIGHT: f32 = 16.0;

const CUTTER_DIAMETER_FLAT: f32 = 10.0;
const CUTTER_RADIUS_FLAT: f32 = 0.5 * CUTTER_DIAMETER_FLAT;
const CUTTER_HEIGHT_FLAT: f32 = 4.0 * CUTTER_DIAMETER_FLAT;
const FLAT_EPS: f32 = 0.1 * CUTTER_RADIUS_FLAT;

const CUTTER_DIAMETER_DETAIL: f32 = 8.0;
pub const CUTTER_RADIUS_DETAIL: f32 = 0.5 * CUTTER_DIAMETER_DETAIL;

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

pub fn detail(model: &Model) -> cncp::Program {
    const CUTTER_DIAMETER: f32 = 8.0;
    const CUTTER_HEIGHT: f32 = 4.0 * CUTTER_DIAMETER;

    let start = std::time::Instant::now();

    let mut locs = initial_locations();
    std::thread::scope(|scope| {
        let grill_thread = scope.spawn(|| grill(model));
        let intersections = model.find_model_intersections();
        let elevated_silhouette = model.elevated_silhouette().unwrap();

        std::thread::scope(|scope| {
            let sand_thread = scope.spawn(|| sand(&intersections, model));
            let inters_thread = scope.spawn(|| inters(&intersections, &elevated_silhouette));

            let sand = sand_thread.join().unwrap();
            locs.extend(sand);

            let inters = inters_thread.join().unwrap();
            locs.extend(inters);
        });

        let grill = grill_thread.join().unwrap();
        locs.extend(grill);
    });
    let end = std::time::Instant::now();

    add_ending_locs(&mut locs);

    println!("Time: {}", (end - start).as_secs_f32());

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

    for hole in holes.iter() {
        let mut first_high = wrld_to_mod(&hole.points[0].point.xzy().coords);
        first_high.z = SAFE_HEIGHT;
        locs.push(first_high);

        let contour = grill_contour(hole);

        locs.extend(grill_net(&contour));
        locs.extend(
            grill_net(&contour.iter().map(|p| p.yxz()).collect_vec())
                .iter()
                .map(|p| p.yxz()),
        );

        let mut first_high = *contour.first().unwrap();
        first_high.z = SAFE_HEIGHT;
        locs.push(first_high);

        locs.extend(contour);

        let mut last_high = *locs.last().unwrap();
        last_high.z = SAFE_HEIGHT;
        locs.push(last_high);
    }

    locs
}

fn grill_contour(hole: &Intersection) -> Vec<Vector3<f32>> {
    let len = hole.points.len();
    hole.points
        .iter()
        .map(|p| {
            let mut at_base = wrld_to_mod(&p.point.coords);
            at_base.z = BASE_HEIGHT;
            at_base
        })
        .cycle()
        .take(len + SAFE_CONTOUR_ADD) // to make sure that the whole hole is milled
        .collect()
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
    let paths = (3.5 * span / CUTTER_RADIUS_DETAIL).ceil() as i32;
    let x_step = span / paths as f32;

    let mut x = min_x;

    let [mut first_safe, _] = grill_point_pair(0, x, &x_map);
    first_safe.z = SAFE_HEIGHT;
    locs.push(first_safe);

    for i in 0..paths {
        let points = grill_point_pair(i, x, &x_map);
        locs.extend(points);

        x += x_step;
    }

    let mut last_safe = *locs.last().unwrap();
    last_safe.z = SAFE_HEIGHT;
    locs.push(last_safe);

    locs
}

fn grill_point_pair(
    i: i32,
    x: NotNan<f32>,
    x_map: &BTreeMap<NotNan<f32>, f32>,
) -> [Vector3<f32>; 2] {
    let range = x - INTERSECTION_IN_BLOCK..x + INTERSECTION_IN_BLOCK;
    let vals = x_map.range(range).map(|(_, v)| *v);
    let len = vals.clone().count();
    let average: f32 = vals.clone().sum::<f32>() / len as f32;

    let max_y = vals
        .clone()
        .filter(|v| *v > average)
        .fold(f32::INFINITY, f32::min);

    let min_y = vals
        .filter(|v| *v <= average)
        .fold(-f32::INFINITY, f32::max);

    if i % 2 == 0 {
        [
            vector![*x, min_y, BASE_HEIGHT],
            vector![*x, max_y, BASE_HEIGHT],
        ]
    } else {
        [
            vector![*x, max_y, BASE_HEIGHT],
            vector![*x, min_y, BASE_HEIGHT],
        ]
    }
}

fn sand(intersections: &[Intersection; INTERSECTIONS.len()], model: &Model) -> Vec<Vector3<f32>> {
    let mut locs = Vec::new();

    sand_shackle(Side::Left, intersections, model, &mut locs);
    sand_shackle(Side::Right, intersections, model, &mut locs);
    sand_shield(Side::Left, intersections, model, &mut locs);
    sand_shield(Side::Right, intersections, model, &mut locs);
    sand_screw(Side::Left, intersections, model, &mut locs);
    sand_screw(Side::Right, intersections, model, &mut locs);
    sand_body(intersections, model, &mut locs);

    locs
}

fn extend_sand(locs: &mut Vec<Vector3<f32>>, extension: Vec<Vector3<f32>>) {
    let Some(mut first_safe) = extension.first().copied() else {
        return;
    };
    first_safe.z = SAFE_HEIGHT;

    let Some(mut last_safe) = extension.last().copied() else {
        return;
    };
    last_safe.z = SAFE_HEIGHT;

    locs.push(first_safe);
    locs.extend(extension);
    locs.push(last_safe);
}

enum Side {
    Left,
    Right,
}

fn sand_shackle(
    shackle: Side,
    intersections: &[Intersection; INTERSECTIONS.len()],
    model: &Model,
    locs: &mut Vec<Vector3<f32>>,
) {
    const U_STEP: f64 = 0.012;
    const V_STEP: f64 = 0.005;

    let surface = match shackle {
        Side::Left => model.surfaces[&LEFT_SHACKLE_ID].as_ref(),
        Side::Right => model.surfaces[&RIGHT_SHACKLE_ID].as_ref(),
    };

    let inters = match shackle {
        Side::Left => [
            &intersections[LEFT_SHACKLE_INTERS[0]],
            &intersections[LEFT_SHACKLE_INTERS[1]],
        ],
        Side::Right => [
            &intersections[RIGHT_SHACKLE_INTERS[0]],
            &intersections[RIGHT_SHACKLE_INTERS[1]],
        ],
    };

    extend_sand(
        locs,
        sand_element(
            &inters,
            surface,
            NotNan::new(U_STEP).unwrap(),
            NotNan::new(V_STEP).unwrap(),
            false,
            (
                -NotNan::new(f64::INFINITY).unwrap(),
                NotNan::new(f64::INFINITY).unwrap(),
            ),
            (
                -NotNan::new(f64::INFINITY).unwrap(),
                NotNan::new(f64::INFINITY).unwrap(),
            ),
            BASE_HEIGHT + 20.0,
            None,
        ),
    );
}

fn sand_shield(
    shield: Side,
    intersections: &[Intersection; INTERSECTIONS.len()],
    model: &Model,
    locs: &mut Vec<Vector3<f32>>,
) {
    const U_STEP: f64 = 0.017;
    const V_STEP: f64 = 0.017;

    let surface = match shield {
        Side::Left => model.surfaces[&LEFT_SHIELD_ID].as_ref(),
        Side::Right => model.surfaces[&RIGHT_SHIELD_ID].as_ref(),
    };

    let mut inverted = MaybeUninit::uninit();

    let inters = match shield {
        Side::Left => {
            inverted.write(intersections[LEFT_SCREW_INTER].inverted());
            [&intersections[LEFT_SHIELD_INTER], unsafe {
                inverted.assume_init_ref()
            }]
        }
        Side::Right => {
            inverted.write(intersections[RIGHT_SCREW_INTER].inverted());
            [&intersections[RIGHT_SHIELD_INTER], unsafe {
                inverted.assume_init_ref()
            }]
        }
    };

    extend_sand(
        locs,
        sand_element(
            &inters,
            surface,
            NotNan::new(U_STEP).unwrap(),
            NotNan::new(V_STEP).unwrap(),
            false,
            (
                -NotNan::new(f64::INFINITY).unwrap(),
                NotNan::new(f64::INFINITY).unwrap(),
            ),
            (
                -NotNan::new(f64::INFINITY).unwrap(),
                NotNan::new(0.5).unwrap(),
            ),
            SAFE_HEIGHT,
            None,
        ),
    );
    extend_sand(
        locs,
        sand_element(
            &inters,
            surface,
            NotNan::new(U_STEP).unwrap(),
            NotNan::new(V_STEP).unwrap(),
            false,
            (
                -NotNan::new(f64::INFINITY).unwrap(),
                NotNan::new(f64::INFINITY).unwrap(),
            ),
            (
                NotNan::new(0.5).unwrap(),
                NotNan::new(f64::INFINITY).unwrap(),
            ),
            SAFE_HEIGHT,
            None,
        ),
    );
}

fn sand_screw(
    screw: Side,
    intersections: &[Intersection; INTERSECTIONS.len()],
    model: &Model,
    locs: &mut Vec<Vector3<f32>>,
) {
    const U_STEP: f64 = 0.005;
    const V_STEP: f64 = 0.005;

    let surface = match screw {
        Side::Left => model.surfaces[&LEFT_SCREW_ID].as_ref(),
        Side::Right => model.surfaces[&RIGHT_SCREW_ID].as_ref(),
    };

    let inters = match screw {
        Side::Left => [&intersections[LEFT_SCREW_INTER]],
        Side::Right => [&intersections[RIGHT_SCREW_INTER]],
    };

    extend_sand(
        locs,
        sand_element(
            &inters,
            surface,
            NotNan::new(U_STEP).unwrap(),
            NotNan::new(V_STEP).unwrap(),
            true,
            (
                -NotNan::new(f64::INFINITY).unwrap(),
                NotNan::new(f64::INFINITY).unwrap(),
            ),
            (
                -NotNan::new(f64::INFINITY).unwrap(),
                NotNan::new(f64::INFINITY).unwrap(),
            ),
            SAFE_HEIGHT,
            None,
        ),
    );
}

fn sand_body(
    intersections: &[Intersection; INTERSECTIONS.len()],
    model: &Model,
    locs: &mut Vec<Vector3<f32>>,
) {
    const U_STEP: f64 = 0.005;
    const V_STEP: f64 = 0.005;

    let surface = model.surfaces[&BODY_ID].as_ref();

    let i0 = intersections[LEFT_SHACKLE_INTERS[0]].inverted();
    let i1 = intersections[LEFT_SHACKLE_INTERS[1]].inverted();
    let i2 = intersections[RIGHT_SHACKLE_INTERS[0]].inverted();
    let i3 = intersections[RIGHT_SHACKLE_INTERS[1]].inverted();
    let i4 = intersections[LEFT_SHIELD_INTER].inverted();
    let i5 = intersections[RIGHT_SHIELD_INTER].inverted();

    let inters = [&i0, &i1, &i2, &i3, &i4, &i5];

    let u_bounds = [0.0, 0.33, 0.66].map(|n| NotNan::new(n).unwrap());
    let v_bounds = [0.0, 0.20, 0.40, 0.60, 0.80, 1.0].map(|n| NotNan::new(n).unwrap());
    let u_axes = [0.1, 0.25, 0.50, 0.75, 0.9];

    for u_bound in u_bounds
        .iter()
        .copied()
        .tuple_windows::<(NotNan<f64>, NotNan<f64>)>()
    {
        for (v_bound, u_axis) in v_bounds
            .iter()
            .copied()
            .tuple_windows::<(NotNan<f64>, NotNan<f64>)>()
            .zip(u_axes.iter())
        {
            extend_sand(
                locs,
                sand_element(
                    &inters,
                    surface,
                    NotNan::new(U_STEP).unwrap(),
                    NotNan::new(V_STEP).unwrap(),
                    false,
                    u_bound,
                    v_bound,
                    BASE_HEIGHT + 1.0,
                    Some(*u_axis),
                ),
            );
        }
    }
}

fn sand_element(
    inters: &[&Intersection],
    surface: &dyn DifferentialParametricForm<2, 3>,
    u_step: NotNan<f64>,
    v_step: NotNan<f64>,
    invert_surface: bool,
    u_bound: (NotNan<f64>, NotNan<f64>),
    v_bound: (NotNan<f64>, NotNan<f64>),
    safe_break: f32,
    u_axis: Option<f64>,
) -> Vec<Vector3<f32>> {
    let mut locs = Vec::<Vector3<f32>>::new();
    let multiplier = if invert_surface { -1.0 } else { 1.0 };

    let shifted_sufrace = ShiftedSurface::new(
        surface,
        multiplier * (CUTTER_RADIUS_DETAIL / MODEL_SCALE) as f64,
    );

    let btree_u: BTreeMap<_, _> = inters
        .iter()
        .flat_map(|i| &i.points)
        .filter(|p| *u_bound.0 <= p.surface_1.x && p.surface_1.x <= *u_bound.1)
        .filter(|p| *v_bound.0 <= p.surface_1.y && p.surface_1.y <= *v_bound.1)
        .map(|p| (NotNan::new(p.surface_1.x).unwrap(), p))
        .collect();

    let min_u = if u_bound.0.is_finite() {
        u_bound.0
    } else {
        *btree_u
            .first_key_value()
            .map(|p| p.0)
            .unwrap_or(&u_bound.0)
            .clamp(&u_bound.0, &u_bound.1)
    };

    let max_u = if u_bound.1.is_finite() {
        u_bound.1
    } else {
        *btree_u
            .last_key_value()
            .map(|p| p.0)
            .unwrap_or(&u_bound.1)
            .clamp(&u_bound.0, &u_bound.1)
    };

    let mut break_occured = false;
    let u_pillow = u_step * 0.75;
    let mut u = min_u + u_pillow;
    let mut reverse = false;
    while u <= max_u {
        let Some((min_v, max_v)) = min_max_v(u, u_step, &btree_u, v_bound, u_axis) else {
            u += u_step;
            continue;
        };

        let v_pillow = *v_step * 0.25;

        let min_v = min_v.clamp(*v_bound.0, *v_bound.1) + v_pillow;
        let max_v = max_v.clamp(*v_bound.0, *v_bound.1) - v_pillow;

        let mut v = if !reverse { min_v } else { max_v };
        while min_v <= v && v <= max_v {
            let value = shifted_sufrace.value(&vector![*u, v]);
            let mod_value = wrld_to_mod(&value.coords) - vector![0.0, 0.0, CUTTER_RADIUS_DETAIL];

            if mod_value.z < BASE_HEIGHT {
                if !break_occured && locs.last().is_some() {
                    let mut last_safe = *locs.last().unwrap();
                    last_safe.z = safe_break;
                    locs.push(last_safe);
                    break_occured = true;
                }
            } else {
                if break_occured {
                    let mut first_safe = mod_value;
                    first_safe.z = safe_break;
                    locs.push(first_safe);
                    break_occured = false;
                }

                locs.push(mod_value);
            }

            // Make sure both both limits are accounted for
            let clamp = min_v < v && v < max_v;
            v += if !reverse { *v_step } else { -*v_step };
            if clamp {
                v = v.clamp(min_v, max_v);
            }
        }

        u += u_step;
        reverse = !reverse;
    }

    locs
}

fn min_max_v(
    u: NotNan<f64>,
    u_step: NotNan<f64>,
    btree_u: &BTreeMap<NotNan<f64>, &IntersectionPoint>,
    v_bound: (NotNan<f64>, NotNan<f64>),
    u_axis: Option<f64>,
) -> Option<(f64, f64)> {
    let pivot = if let Some(u_axis) = u_axis {
        u_axis
    } else {
        let avg_range = btree_u.range((u - u_step * 4.0)..(u + u_step * 4.0));
        let len = avg_range.clone().count();
        avg_range.clone().map(|p| p.1.surface_1.y).sum::<f64>() / len as f64
    };

    let max_v = extr_v(u, u_step, btree_u, v_bound.1, |p| p.1.surface_1.y >= pivot)?;
    let min_v = extr_v(u, u_step, btree_u, v_bound.0, |p| p.1.surface_1.y <= pivot)?;

    Some((min_v, max_v))
}

fn extr_v<F: FnMut(&(&NotNan<f64>, &&IntersectionPoint)) -> bool + Copy>(
    u: NotNan<f64>,
    u_step: NotNan<f64>,
    btree_u: &BTreeMap<NotNan<f64>, &IntersectionPoint>,
    v_bound: NotNan<f64>,
    filter: F,
) -> Option<f64> {
    const DIST_TOLERANCE: f64 = 1.0;

    let lower_bound_u = btree_u.range(..u).filter(filter).max_by_key(|(k, _)| *k);
    let upper_bound_u = btree_u.range(u..).filter(filter).min_by_key(|(k, _)| *k);

    let Some(lower_bound_u) = lower_bound_u.or(upper_bound_u) else {
        return Some(*v_bound);
    };
    let upper_bound_u = upper_bound_u.unwrap_or(lower_bound_u);

    Some(
        if v_bound.is_finite()
            && (upper_bound_u.1.surface_1.x - lower_bound_u.1.surface_1.x
                > DIST_TOLERANCE * *u_step
                || (upper_bound_u.1.surface_1.x - *u).abs() > DIST_TOLERANCE * *u_step
                || (lower_bound_u.1.surface_1.x - *u).abs() > DIST_TOLERANCE * *u_step)
        {
            *v_bound
        } else {
            let range_high = upper_bound_u.0 - lower_bound_u.0;
            let interpol = if range_high == 0.0 {
                NotNan::new(0.0).unwrap()
            } else {
                (u - lower_bound_u.0) / range_high
            };

            lower_bound_u.1.surface_1.y * *interpol
                + upper_bound_u.1.surface_1.y * (1.0 - *interpol)
        },
    )
}

fn wrld_to_mod(vec: &Vector3<f64>) -> Vector3<f32> {
    let mut v = vec_64_to_32(vec - PLANE_CENTER).xzy() * MODEL_SCALE;
    v.z += BASE_HEIGHT;
    v
}

fn inters(
    intersections: &[Intersection; INTERSECTIONS.len()],
    elevated_silhouette: &Intersection,
) -> Vec<Vector3<f32>> {
    let mut locs = Vec::new();

    for intersection in intersections.iter().chain([elevated_silhouette]) {
        let mut initial_locs = intersection
            .points
            .iter()
            .map(|p| wrld_to_mod(&p.point.coords) - vector![0.0, 0.0, CUTTER_RADIUS_DETAIL])
            .collect_vec();

        if let Some(first_under) = initial_locs.iter().position(|p| p.z < BASE_HEIGHT) {
            let first_over = initial_locs
                .iter()
                .skip(first_under + 1)
                .position(|p| p.z >= BASE_HEIGHT)
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

pub fn signa() -> cncp::Program {
    const CUTTER_DIAMETER: f32 = 1.0;
    const CUTTER_HEIGHT: f32 = 4.0 * CUTTER_DIAMETER;
    const TEXT: &str = "szymon\n\rzygul\x08{a";
    const POS: Vector3<f32> = vector![50.0, -60.0, 0.0];

    let mut locs = initial_locations();

    locs.extend(svg::parse_signature(TEXT, &POS));

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

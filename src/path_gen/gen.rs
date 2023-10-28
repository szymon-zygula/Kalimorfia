use super::model::{Model, BLOCK_SIZE};
use crate::cnc::{
    block::Block,
    mill::{Cutter, CutterShape},
    program as cncp,
};
use nalgebra::{vector, Vector3};

const SAFE_HEIGHT: f32 = 66.0;

pub fn rough(model: &Model) -> cncp::Program {
    const UPPER_PLANE_HEIGHT: f32 = 35.0;
    const LOWER_PLANE_HEIGHT: f32 = 20.0;
    const CUTTER_DIAMETER: f32 = 16.0;
    const CUTTER_HEIGHT: f32 = 4.0 * CUTTER_DIAMETER;
    const SPACING: f32 = CUTTER_DIAMETER * 0.5;
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
            diameter: CUTTER_DIAMETER,
            shape: CutterShape::Ball,
        },
    )
}

fn rough_plane(height: f32, heightmap: &Block, spacing: f32, sampling: f32) -> Vec<Vector3<f32>> {
    let mut reverse = false;
    let mut locs = Vec::new();
    let mut x = 0.5 * BLOCK_SIZE + spacing;

    for _ in 0..(BLOCK_SIZE / spacing + 4.0) as usize {
        let mut line = rough_line(height, x, heightmap, spacing, sampling);
        if reverse {
            line.reverse();
        }

        locs.extend(line);
        x -= spacing;
        reverse = !reverse;
    }

    locs
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
            f32::max(
                heightmap.height(bx.floor() as usize, by.floor() as usize),
                height,
            )
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

pub fn flat(model: &Model) -> cncp::Program {
    const CUTTER_DIAMETER: f32 = 10.0;
    const CUTTER_HEIGHT: f32 = 4.0 * CUTTER_DIAMETER;

    let mut locs = initial_locations();
    add_ending_locs(&mut locs);

    cncp::Program::from_locations(
        locs,
        Cutter {
            height: CUTTER_HEIGHT,
            diameter: CUTTER_DIAMETER,
            shape: CutterShape::Cylinder,
        },
    )
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

use super::model::Model;
use crate::cnc::program as cncp;
use nalgebra::{vector, Vector3};

fn rough(model: &Model) -> cncp::Program {
    let locs = initial_locations();
    todo!()
}

fn flat(model: &Model) -> cncp::Program {
    let locs = initial_locations();
    todo!()
}

fn detail(model: &Model) -> cncp::Program {
    let locs = initial_locations();
    todo!()
}

fn signa(model: &Model) -> cncp::Program {
    let locs = initial_locations();
    todo!()
}

fn initial_locations() -> Vec<Vector3<f32>> {
    vec![vector![0.0, 0.0, 66.0]]
}

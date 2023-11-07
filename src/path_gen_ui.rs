use crate::{main_control::MainControl, state::State};
use kalimorfia::{
    cnc::{
        block::Block, mill::Mill, milling_player::MillingPlayer, milling_process::MillingProcess,
    },
    entities::cnc_block::{CNCBlock, CNCBlockArgs},
    path_gen::gen::*,
    path_gen::model::*,
};
use nalgebra::vector;
use std::path::Path;
use std::rc::Rc;

const SAVE_PATH: &str = "gen-paths";
const TEST_SAMPLING: i32 = 1500;

pub fn path_gen_ui(ui: &imgui::Ui, state: &mut State, control: &mut MainControl) {
    ui.window("Path generation control")
        .size([500.0, 300.0], imgui::Condition::FirstUseEver)
        .position([500.0, 0.0], imgui::Condition::FirstUseEver)
        .build(|| {
            ui.separator();
            ui.text("Generation");
            ui.separator();

            let mut add_block = false;

            if ui.button("Rough paths") {
                rough(&get_model(state, control))
                    .save_to_file(Path::new(&format!("{SAVE_PATH}/1.k16")));
                add_block = true;
            }

            if ui.button("Flat paths") {
                if let Some(prog) = flat(&get_model(state, control)) {
                    prog.save_to_file(Path::new(&format!("{SAVE_PATH}/2.f10")));
                } else {
                    println!("Failed to find flat paths -- try again");
                }

                add_block = true;
            }

            if ui.button("Detailed paths") {
                detail(&get_model(state, control))
                    .save_to_file(Path::new(&format!("{SAVE_PATH}/3.k08")));
                add_block = true;
            }

            if ui.button("Signature paths") {
                signa().save_to_file(Path::new(&format!("{SAVE_PATH}/4.k01")));
                add_block = true;
            }

            if add_block {
                control.add_cnc_block(
                    state,
                    CNCBlockArgs {
                        size: vector![BLOCK_SIZE, BLOCK_SIZE, BLOCK_HEIGHT],
                        sampling: vector![TEST_SAMPLING, TEST_SAMPLING],
                    },
                );
            }

            ui.separator();
            ui.text("Tests");
            ui.separator();

            if ui.button("Silhouette") {
                test_silhouette(state, control);
            }

            if ui.button("Elevated silhouette") {
                test_elevated_silhouette(state, control);
            }

            if ui.button("Heightmap") {
                test_heightmap(state, control);
            }

            if ui.button("Intersections") {
                test_intersections(state, control);
            }

            if ui.button("Holes") {
                test_holes(state, control);
            }

            if ui.button("Rough-Flat and save Detailed") {
                test_rough_flat(state, control);
            }
        });
}

fn get_model(state: &mut State, control: &mut MainControl) -> Model {
    let manager = control.entity_manager.borrow();
    let (targets, ids) = state
        .selector
        .selected()
        .iter()
        .copied()
        .filter_map(|id| manager.get_entity(id).as_parametric_2_to_3().zip(Some(id)))
        .unzip();

    Model::new(targets, ids)
}

fn test_silhouette(state: &mut State, control: &mut MainControl) {
    let model = get_model(state, control);
    let Some(intersection) = model.silhouette() else {
        println!("Model has no intersection with the XZ plane");
        return;
    };
    control.add_intersection_curve(state, intersection);
}

fn test_elevated_silhouette(state: &mut State, control: &mut MainControl) {
    let model = get_model(state, control);
    let Some(intersection) = model.elevated_silhouette() else {
        println!("Model has no intersection with the XZ plane");
        return;
    };
    control.add_intersection_curve(state, intersection);
}

fn test_heightmap(state: &mut State, control: &mut MainControl) {
    let model = get_model(state, control);
    let block = model.sampled_block();
    let entity_block = Box::new(CNCBlock::with_block(
        control.gl,
        Rc::clone(&state.name_repo),
        Rc::clone(&control.shader_manager),
        block,
    ));
    let id = control.entity_manager.borrow_mut().add_entity(entity_block);
    state.selector.add_selectable(id);
}

fn test_intersections(state: &mut State, control: &mut MainControl) {
    let model = get_model(state, control);
    for intersection in model.find_model_intersections() {
        control.add_intersection_curve(state, intersection);
    }
}

fn test_holes(state: &mut State, control: &mut MainControl) {
    let model = get_model(state, control);
    for intersection in model.find_holes() {
        control.add_intersection_curve(state, intersection);
    }
}

fn test_rough_flat(state: &mut State, control: &mut MainControl) {
    let block = Block::new(
        vector![TEST_SAMPLING as usize, TEST_SAMPLING as usize],
        vector![BLOCK_SIZE, BLOCK_SIZE, BLOCK_HEIGHT],
    );

    let model = get_model(state, control);
    let rough = rough(&model);
    rough.save_to_file(Path::new(&format!("{SAVE_PATH}/1.k16")));
    let flat = flat(&model).expect("Flat milling failed");
    rough.save_to_file(Path::new(&format!("{SAVE_PATH}/2.f10")));

    println!("Rough paths");
    let mut mill = Mill::new(rough.shape());
    mill.move_to(vector![0.0, 0.0, SAFE_HEIGHT]).unwrap();
    let process = MillingProcess::new(mill, rough, block);
    let mut player = MillingPlayer::new(process);
    player.complete().expect("Milling error");
    let (_, _, block) = player.take().retake_all();

    println!("Flat paths");
    let mut mill = Mill::new(flat.shape());
    mill.move_to(vector![0.0, 0.0, SAFE_HEIGHT]).unwrap();
    let process = MillingProcess::new(mill, flat, block);
    let mut player = MillingPlayer::new(process);
    player.complete().expect("Milling error");
    let (_, _, block) = player.take().retake_all();

    let block = Box::new(CNCBlock::with_block(
        control.gl,
        Rc::clone(&state.name_repo),
        Rc::clone(&control.shader_manager),
        block,
    ));

    let id = control.entity_manager.borrow_mut().add_entity(block);
    state.selector.add_selectable(id);

    detail(&get_model(state, control)).save_to_file(Path::new(&format!("{SAVE_PATH}/3.k08")));
}

use crate::{main_control::MainControl, state::State};
use kalimorfia::{
    entities::cnc_block::{CNCBlock, CNCBlockArgs},
    path_gen::gen::*,
    path_gen::model::*,
};
use nalgebra::vector;
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
                    .save_to_file(std::path::Path::new(&format!("{SAVE_PATH}/1.k16")));
                add_block = true;
            }

            if ui.button("Flat paths") {
                if let Some(prog) = flat(&get_model(state, control)) {
                    prog.save_to_file(std::path::Path::new(&format!("{SAVE_PATH}/2.f10")));
                } else {
                    println!("Failed to find flat paths -- try again");
                }

                add_block = true;
            }

            if ui.button("Detailed paths") {
                detail(&get_model(state, control))
                    .save_to_file(std::path::Path::new(&format!("{SAVE_PATH}/3.k08")));
                add_block = true;
            }

            if ui.button("Signature paths") {
                signa(&get_model(state, control))
                    .save_to_file(std::path::Path::new(&format!("{SAVE_PATH}/4.k01")));
                add_block = true;
            }

            if add_block {
                let id = control.add_cnc_block(
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

            if ui.button("Find silhouette") {
                test_silhouette(state, control);
            }

            if ui.button("Heightmap") {
                test_heightmap(state, control);
            }
        });
}

fn get_model(state: &mut State, control: &mut MainControl) -> Model {
    let manager = control.entity_manager.borrow();
    let targets: Vec<_> = state
        .selector
        .selected()
        .iter()
        .copied()
        .filter_map(|id| manager.get_entity(id).as_parametric_2_to_3())
        .collect();

    Model::new(targets)
}

fn test_silhouette(state: &mut State, control: &mut MainControl) {
    let model = get_model(state, control);
    let Some(intersection) = model.silhouette() else {
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

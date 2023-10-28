use crate::{main_control::MainControl, state::State};
use kalimorfia::{entities::cnc_block::CNCBlock, path_gen::model::Model};
use std::rc::Rc;

pub fn path_gen_ui(ui: &imgui::Ui, state: &mut State, control: &mut MainControl) {
    ui.window("Path generation control")
        .size([500.0, 300.0], imgui::Condition::FirstUseEver)
        .position([500.0, 0.0], imgui::Condition::FirstUseEver)
        .build(|| {
            ui.separator();
            ui.text("Generation");
            ui.separator();

            if ui.button("Rough paths") {

            }

            if ui.button("Flat paths") {

            }
            
            if ui.button("Detailed paths") {

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

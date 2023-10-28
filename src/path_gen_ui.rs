use crate::{main_control::MainControl, state::State};
use kalimorfia::path_gen::model::Model;

pub fn path_gen_ui(ui: &imgui::Ui, state: &mut State, control: &mut MainControl) {
    ui.window("Path generation control")
        .size([500.0, 300.0], imgui::Condition::FirstUseEver)
        .position([500.0, 0.0], imgui::Condition::FirstUseEver)
        .build(|| {
            ui.separator();
            if ui.button("Find silhouette") {
                test_silhouette(state, control);
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

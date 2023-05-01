use crate::state::State;
use kalimorfia::{
    camera::Stereo,
    entities::{
        cubic_spline_c0::CubicSplineC0,
        cubic_spline_c2::CubicSplineC2,
        entity::{Entity, ReferentialSceneEntity, SceneObject},
        interpolating_spline::InterpolatingSpline,
        manager::EntityManager,
        point::Point,
        torus::Torus,
    },
    render::shader_manager::ShaderManager,
    ui::selector::Selector,
};
use std::{cell::RefCell, rc::Rc};

pub struct MainControl<'gl, 'a> {
    entity_manager: &'a RefCell<EntityManager<'gl>>,
    shader_manager: Rc<ShaderManager<'gl>>,
    gl: &'gl glow::Context,
}

impl<'gl, 'a> MainControl<'gl, 'a> {
    pub fn new(
        shader_manager: Rc<ShaderManager<'gl>>,
        entity_manager: &'a RefCell<EntityManager<'gl>>,
        gl: &'gl glow::Context,
    ) -> Self {
        Self {
            entity_manager,
            gl,
            shader_manager,
        }
    }

    pub fn build_ui(&self, ui: &mut imgui::Ui, state: &mut State<'gl, '_>) {
        self.main_control_window(ui, state);
        self.selection_window(ui, state);
    }

    fn main_control_window(&self, ui: &imgui::Ui, state: &mut State) {
        ui.window("Main control")
            .size([500.0, 300.0], imgui::Condition::FirstUseEver)
            .position([0.0, 0.0], imgui::Condition::FirstUseEver)
            .build(|| {
                ui.separator();
                self.cursor_control(ui, state);
                ui.separator();
                self.stereoscopy_control(ui, state);
                ui.separator();
                self.object_creation(ui, state);
                ui.separator();
                state.selector.control_ui(ui, self.entity_manager);
            });
    }

    fn selection_window(&self, ui: &imgui::Ui, state: &mut State) {
        let _token = ui.push_id("selection_window");
        ui.window("Selection")
            .size([500.0, 500.0], imgui::Condition::FirstUseEver)
            .position([0.0, 300.0], imgui::Condition::FirstUseEver)
            .build(|| {
                ui.separator();
                self.entity_manager
                    .borrow_mut()
                    .control_referential_ui(state.selected_aggregate_id, ui);
            });
    }

    fn cursor_control(&self, ui: &imgui::Ui, state: &mut State) {
        state.cursor.control_ui(ui);

        if ui.button("Center on cursor") {
            state.camera.center = state.cursor.location().unwrap();
        }
    }

    fn stereoscopy_control(&self, ui: &imgui::Ui, state: &mut State) {
        let _token = ui.push_id("stereoscopy");
        let mut stereoscopy = state.camera.stereo.is_some();

        if ui.checkbox("Stereoscopy", &mut stereoscopy) {
            state.camera.stereo = if stereoscopy {
                Some(Stereo::new())
            } else {
                None
            };
        }

        if let Some(stereo) = &mut state.camera.stereo {
            ui.slider_config("Baseline", 0.1, 0.5)
                .flags(imgui::SliderFlags::NO_INPUT)
                .build(&mut stereo.baseline);

            ui.slider_config("Screen distance", 10.0, 1.0)
                .flags(imgui::SliderFlags::NO_INPUT)
                .build(&mut stereo.screen_distance);
        }
    }

    fn object_creation(&self, ui: &imgui::Ui, state: &mut State) {
        ui.text("Object creation");
        ui.columns(3, "creation_columns", false);
        if ui.button("Torus") {
            self.add_torus(state);
        }

        ui.next_column();
        if ui.button("Point") {
            self.add_point(state);
        }

        ui.next_column();
        if ui.button("Cubic spline C0") {
            self.add_cubic_spline_c0(state);
        }

        ui.next_column();
        if ui.button("Cubic spline C2") {
            self.add_cubic_spline_c2(state);
        }

        ui.next_column();
        if ui.button("Interpolating spline") {
            self.add_interpolating_spline(state);
        }

        ui.next_column();
        ui.columns(1, "clear_columns", false);
    }

    fn add_point(&self, state: &mut State) {
        let point = Box::new(Point::with_position(
            self.gl,
            state.cursor.location().unwrap(),
            Rc::clone(&state.name_repo),
            Rc::clone(&self.shader_manager),
        ));

        let id = self.entity_manager.borrow_mut().add_entity(point);
        state.selector.add_selectable(id);

        if let Some(only_id) = state.selector.only_selected() {
            if self.entity_manager.borrow().entities()[&only_id]
                .borrow_mut()
                .add_point(id, self.entity_manager.borrow().entities())
            {
                self.entity_manager.borrow_mut().subscribe(only_id, id);
            }
        }
    }

    fn add_torus(&self, state: &mut State) {
        let id = self
            .entity_manager
            .borrow_mut()
            .add_entity(Box::new(Torus::with_position(
                self.gl,
                state.cursor.location().unwrap(),
                Rc::clone(&state.name_repo),
                Rc::clone(&self.shader_manager),
            )));
        state.selector.add_selectable(id);
    }

    fn add_cubic_spline_c0(&self, state: &mut State) {
        let selected_points = self.selected_points(&state.selector);
        let spline = CubicSplineC0::through_points(
            self.gl,
            Rc::clone(&state.name_repo),
            Rc::clone(&self.shader_manager),
            selected_points,
            self.entity_manager.borrow().entities(),
        );

        self.add_spline(state, spline);
    }

    fn add_cubic_spline_c2(&self, state: &mut State) {
        let selected_points = self.selected_points(&state.selector);
        let spline = CubicSplineC2::through_points(
            self.gl,
            Rc::clone(&state.name_repo),
            Rc::clone(&self.shader_manager),
            selected_points,
            self.entity_manager.borrow().entities(),
        );

        self.add_spline(state, spline);
    }

    fn add_interpolating_spline(&self, state: &mut State) {
        let selected_points = self.selected_points(&state.selector);
        let spline = InterpolatingSpline::through_points(
            self.gl,
            Rc::clone(&state.name_repo),
            Rc::clone(&self.shader_manager),
            selected_points,
            self.entity_manager.borrow().entities(),
        );

        self.add_spline(state, spline);
    }

    fn add_spline<T: ReferentialSceneEntity<'gl> + 'gl>(&self, state: &mut State, spline: T) {
        let boxed_spline = Box::new(spline);

        let id = self.entity_manager.borrow_mut().add_entity(boxed_spline);

        for selected in self.selected_points(&state.selector) {
            self.entity_manager.borrow_mut().subscribe(id, selected);
        }

        state.selector.add_selectable(id);
    }

    fn selected_points(&self, selector: &Selector) -> Vec<usize> {
        let mut selected: Vec<usize> = selector
            .selected()
            .iter()
            .filter(|&&id| {
                self.entity_manager
                    .borrow()
                    .get_entity(id)
                    .is_single_point()
            })
            .copied()
            .collect();
        selected.sort();
        selected
    }
}

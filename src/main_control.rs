use crate::state::State;
use kalimorfia::{
    camera::Stereo,
    entities::{
        basic::{LinearTransformEntity, Translation},
        bezier_surface_args::BezierSurfaceArgs,
        bezier_surface_c0::BezierSurfaceC0,
        bezier_surface_c2::BezierSurfaceC2,
        cubic_spline_c0::CubicSplineC0,
        cubic_spline_c2::CubicSplineC2,
        entity::{Entity, EntityCollection, ReferentialSceneEntity, SceneObject},
        interpolating_spline::InterpolatingSpline,
        manager::EntityManager,
        point::Point,
        torus::Torus,
    },
    render::shader_manager::ShaderManager,
    repositories::NameRepository,
    ui::selector::Selector,
};
use nalgebra::Vector3;
use std::{cell::RefCell, rc::Rc};

enum BezierSurfaceType {
    C0,
    C2,
}

pub struct MainControl<'gl, 'a> {
    entity_manager: &'a RefCell<EntityManager<'gl>>,
    shader_manager: Rc<ShaderManager<'gl>>,
    bezier_surface_args: Option<BezierSurfaceArgs>,
    added_surface_type: Option<BezierSurfaceType>,
    gl: &'gl glow::Context,
}

impl<'gl, 'a> MainControl<'gl, 'a> {
    pub fn new(
        shader_manager: Rc<ShaderManager<'gl>>,
        entity_manager: &'a RefCell<EntityManager<'gl>>,
        gl: &'gl glow::Context,
    ) -> Self {
        Self {
            added_surface_type: None,
            entity_manager,
            gl,
            shader_manager,
            bezier_surface_args: None,
        }
    }

    pub fn build_ui(&mut self, ui: &mut imgui::Ui, state: &mut State<'gl, '_>) {
        self.main_control_window(ui, state);
        self.selection_window(ui, state);

        if self.bezier_surface_args.is_some() {
            match self.added_surface_type {
                Some(BezierSurfaceType::C0) => {
                    self.bezier_surface_window(ui, state, BezierSurfaceC0::new)
                }
                Some(BezierSurfaceType::C2) => {
                    self.bezier_surface_window(ui, state, BezierSurfaceC2::new)
                }
                _ => {}
            }
        }
    }

    fn main_control_window(&mut self, ui: &imgui::Ui, state: &mut State) {
        ui.window("Main control")
            .size([500.0, 550.0], imgui::Condition::FirstUseEver)
            .position([0.0, 0.0], imgui::Condition::FirstUseEver)
            .build(|| {
                ui.separator();
                self.cursor_control(ui, state);
                ui.separator();
                self.stereoscopy_control(ui, state);
                ui.separator();
                self.additional_control(ui, state);
                ui.separator();
                self.object_creation(ui, state);
                ui.separator();
                state.selector.control_ui(ui, self.entity_manager);
            });
    }

    fn selection_window(&self, ui: &imgui::Ui, state: &mut State) {
        let _token = ui.push_id("selection_window");
        ui.window("Selection")
            .size([500.0, 300.0], imgui::Condition::FirstUseEver)
            .position([0.0, 550.0], imgui::Condition::FirstUseEver)
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

        ui.slider_config("Screen distance", 0.2, 5.0)
            .flags(imgui::SliderFlags::NO_INPUT)
            .build(&mut state.camera.screen_distance);

        if let Some(stereo) = &mut state.camera.stereo {
            ui.slider_config("Baseline", 0.01, 0.50)
                .flags(imgui::SliderFlags::NO_INPUT)
                .build(&mut stereo.baseline);
        }
    }

    fn additional_control(&self, ui: &imgui::Ui, state: &mut State) {
        self.select_deselect_all(ui, state);
        ui.next_column();
        self.remove_selected(ui, state);
    }

    fn select_deselect_all(&self, ui: &imgui::Ui, state: &mut State) {
        if ui.button("Select all") {
            state.selector.select_all();
        }

        if ui.button("Deselect all") {
            state.selector.deselect_all();
        }
    }

    fn remove_selected(&self, ui: &imgui::Ui, state: &mut State) {
        if ui.button("Remove all selected") {
            // Remove everything two times to avoid blockage when a blocking parent and its child
            // are both selected
            for id in state.selector.selected() {
                let removed = self.entity_manager.borrow_mut().remove_entity(id).is_none();
                if removed {
                    state.selector.remove(id);
                }
            }

            let mut all_removed = true;
            for id in state.selector.selected() {
                if self.entity_manager.borrow().entities().contains_key(&id) {
                    let removed = self.entity_manager.borrow_mut().remove_entity(id).is_none();
                    all_removed &= removed;
                    if removed {
                        state.selector.remove(id);
                    }
                }
            }

            if !all_removed {
                ui.open_popup("not_all_removed");
            }
        }

        ui.popup("not_all_removed", || {
            ui.text("Some removals were blocked by other entities");
        });
    }

    fn object_creation(&mut self, ui: &imgui::Ui, state: &mut State) {
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
        if ui.button("Bezier surface C0") {
            self.bezier_surface_args = Some(BezierSurfaceArgs::new_surface());
            self.added_surface_type = Some(BezierSurfaceType::C0);
        }

        ui.next_column();
        if ui.button("Bezier surface C2") {
            self.bezier_surface_args = Some(BezierSurfaceArgs::new_surface());
            self.added_surface_type = Some(BezierSurfaceType::C2);
        }

        ui.next_column();
        ui.columns(1, "clear_columns", false);
    }

    /// Adds a point without adding it to the only selected entity if such an entity exists
    fn quietly_add_point(&self, state: &mut State) -> usize {
        let point = Box::new(Point::with_position(
            self.gl,
            state.cursor.location().unwrap(),
            Rc::clone(&state.name_repo),
            Rc::clone(&self.shader_manager),
        ));

        let id = self.entity_manager.borrow_mut().add_entity(point);
        state.selector.add_selectable(id);
        id
    }

    /// Adds a point and adds it to the only currently selected entity if only one entity is
    /// selected
    fn add_point(&self, state: &mut State) {
        let id = self.quietly_add_point(state);

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

    fn add_bezier_surface<T: ReferentialSceneEntity<'gl> + 'gl>(
        &self,
        state: &mut State,
        args: BezierSurfaceArgs,
        surface_creator: impl FnOnce(
            &'gl glow::Context,
            Rc<RefCell<dyn NameRepository>>,
            Rc<ShaderManager<'gl>>,
            Vec<Vec<usize>>,
            &EntityCollection<'gl>,
            BezierSurfaceArgs,
        ) -> T,
    ) {
        let points = match self.added_surface_type {
            Some(BezierSurfaceType::C0) => self.bezier_surface_points_c0(state, args),
            Some(BezierSurfaceType::C2) => self.bezier_surface_points_c2(state, args),
            None => panic!("Should not happen"),
        };

        let surface = Box::new(surface_creator(
            self.gl,
            Rc::clone(&state.name_repo),
            Rc::clone(&self.shader_manager),
            points.clone(),
            self.entity_manager.borrow().entities(),
            args,
        ));

        let id = self.entity_manager.borrow_mut().add_entity(surface);

        for &point in points.iter().flatten() {
            self.entity_manager.borrow_mut().subscribe(id, point);
        }

        state.selector.add_selectable(id);
    }

    fn bezier_surface_points_c2(
        &self,
        state: &mut State,
        args: BezierSurfaceArgs,
    ) -> Vec<Vec<usize>> {
        let (u_points, v_points) = match &args {
            BezierSurfaceArgs::Surface(surface) => (surface.x_patches + 3, surface.z_patches + 3),
            BezierSurfaceArgs::Cylinder(cyllinder) => {
                (cyllinder.around_patches, cyllinder.along_patches + 3)
            }
        };

        let mut add_v_point = |u: i32, v: i32, u_row: &mut Vec<usize>| {
            let id = self.quietly_add_point(state);

            let transform = match &args {
                BezierSurfaceArgs::Surface(surface) => {
                    let mut transform = LinearTransformEntity::new();
                    transform.translation = Translation::with(
                        state.cursor.location().unwrap().coords
                            + Vector3::new(
                                u as f32 / (u_points - 3) as f32 * surface.x_length,
                                0.0,
                                v as f32 / (v_points - 3) as f32 * surface.z_length,
                            ),
                    );
                    transform
                }
                BezierSurfaceArgs::Cylinder(cyllinder) => {
                    let mut transform = LinearTransformEntity::new();
                    let angle = u as f32 / u_points as f32 * std::f32::consts::PI * 2.0;
                    transform.translation = Translation::with(
                        state.cursor.location().unwrap().coords
                            + Vector3::new(
                                angle.cos() * cyllinder.radius,
                                angle.sin() * cyllinder.radius,
                                v as f32 / (v_points - 3) as f32 * cyllinder.length,
                            ),
                    );
                    transform
                }
            };

            self.entity_manager
                .borrow_mut()
                .get_entity_mut(id)
                .set_model_transform(transform);

            u_row.push(id);
        };

        let mut points: Vec<Vec<usize>> = Vec::new();

        for u in 0..u_points {
            points.push(Vec::new());

            for v in 0..v_points {
                add_v_point(u, v, &mut points[u as usize]);
            }
        }

        points
    }

    fn bezier_surface_points_c0(
        &self,
        state: &mut State,
        args: BezierSurfaceArgs,
    ) -> Vec<Vec<usize>> {
        let (u_points, v_points) = match &args {
            BezierSurfaceArgs::Surface(surface) => {
                (surface.x_patches * 3 + 1, surface.z_patches * 3 + 1)
            }
            BezierSurfaceArgs::Cylinder(cyllinder) => (
                cyllinder.around_patches * 3,
                cyllinder.along_patches * 3 + 1,
            ),
        };

        let mut add_v_point = |u: i32, v: i32, u_row: &mut Vec<usize>| {
            let id = self.quietly_add_point(state);

            let transform = match &args {
                BezierSurfaceArgs::Surface(surface) => {
                    let mut transform = LinearTransformEntity::new();
                    transform.translation = Translation::with(
                        state.cursor.location().unwrap().coords
                            + Vector3::new(
                                u as f32 / (u_points - 1) as f32 * surface.x_length,
                                0.0,
                                v as f32 / (v_points - 1) as f32 * surface.z_length,
                            ),
                    );
                    transform
                }
                BezierSurfaceArgs::Cylinder(cyllinder) => {
                    let mut transform = LinearTransformEntity::new();
                    let angle = u as f32 / u_points as f32 * std::f32::consts::PI * 2.0;
                    transform.translation = Translation::with(
                        state.cursor.location().unwrap().coords
                            + Vector3::new(
                                angle.cos() * cyllinder.radius,
                                angle.sin() * cyllinder.radius,
                                v as f32 / (v_points - 1) as f32 * cyllinder.length,
                            ),
                    );
                    transform
                }
            };

            self.entity_manager
                .borrow_mut()
                .get_entity_mut(id)
                .set_model_transform(transform);

            u_row.push(id);
        };

        let mut points: Vec<Vec<usize>> = Vec::new();

        for u in 0..u_points {
            points.push(Vec::new());

            for v in 0..v_points {
                add_v_point(u, v, &mut points[u as usize]);
            }
        }

        points
    }

    fn bezier_surface_window<T: ReferentialSceneEntity<'gl> + 'gl>(
        &mut self,
        ui: &imgui::Ui,
        state: &mut State,
        surface_creator: impl FnMut(
            &'gl glow::Context,
            Rc<RefCell<dyn NameRepository>>,
            Rc<ShaderManager<'gl>>,
            Vec<Vec<usize>>,
            &EntityCollection<'gl>,
            BezierSurfaceArgs,
        ) -> T,
    ) {
        ui.window("Bezier surface creation")
            .size([350.0, 200.0], imgui::Condition::FirstUseEver)
            .position([300.0, 300.0], imgui::Condition::FirstUseEver)
            .build(|| {
                let args = self.bezier_surface_args.as_mut().unwrap();
                let _token = ui.push_id("bezier_creation_window");

                match args {
                    BezierSurfaceArgs::Surface(..) => {
                        if ui.button("Surface") {
                            *args = BezierSurfaceArgs::new_cylinder();
                        }
                    }
                    BezierSurfaceArgs::Cylinder(..) => {
                        if ui.button("Cylinder") {
                            *args = BezierSurfaceArgs::new_surface();
                        }
                    }
                }

                match args {
                    BezierSurfaceArgs::Surface(surface) => {
                        ui.input_int("X patches", &mut surface.x_patches).build();
                        ui.input_int("Z patches", &mut surface.z_patches).build();

                        ui.input_float("X length", &mut surface.x_length).build();
                        ui.input_float("Z length", &mut surface.z_length).build();
                    }
                    BezierSurfaceArgs::Cylinder(cyllinder) => {
                        ui.input_int("Around patches", &mut cyllinder.around_patches)
                            .build();
                        ui.input_int("Along patches", &mut cyllinder.along_patches)
                            .build();

                        ui.input_float("Length", &mut cyllinder.length).build();
                        ui.input_float("Radius", &mut cyllinder.radius).build();

                        if let Some(BezierSurfaceType::C2) = self.added_surface_type {
                            cyllinder.around_patches = std::cmp::max(cyllinder.around_patches, 3);
                        }
                    }
                }

                args.clamp_values();

                ui.columns(2, "bezier_columns", false);
                if ui.button("Ok") {
                    self.add_bezier_surface(
                        state,
                        *self.bezier_surface_args.as_ref().unwrap(),
                        surface_creator,
                    );
                    self.bezier_surface_args = None;
                }

                ui.next_column();
                if ui.button("Cancel") {
                    self.bezier_surface_args = None;
                }

                ui.next_column();
                ui.columns(1, "clear_columns", false);
            });
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

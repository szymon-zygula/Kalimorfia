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

struct BezierSurfaceArgs {
    x_length: f32,
    z_length: f32,

    x_patches: i32,
    z_patches: i32,
}

struct BezierCyllinderArgs {
    length: f32,
    radius: f32,

    around_patches: i32,
    along_patches: i32,
}

enum BezierSurfaceC0Args {
    Surface(BezierSurfaceArgs),
    Cyllinder(BezierCyllinderArgs),
}

impl BezierSurfaceC0Args {
    const MIN_PATCHES: i32 = 1;
    const MAX_PATCHES: i32 = 30;
    const MIN_LENGTH: f32 = 0.1;
    const MAX_LENGTH: f32 = 10.0;

    pub fn new_surface() -> Self {
        Self::Surface(BezierSurfaceArgs {
            x_length: 1.0,
            z_length: 1.0,

            x_patches: 1,
            z_patches: 1,
        })
    }

    pub fn new_cyllinder() -> Self {
        Self::Cyllinder(BezierCyllinderArgs {
            length: 1.0,
            radius: 1.0,
            around_patches: 1,
            along_patches: 1,
        })
    }

    pub fn clamp_values(&mut self) {
        match self {
            BezierSurfaceC0Args::Surface(surface) => {
                Self::clamp_patches(&mut surface.x_patches);
                Self::clamp_patches(&mut surface.z_patches);
                Self::clamp_length(&mut surface.x_length);
                Self::clamp_length(&mut surface.z_length);
            }
            BezierSurfaceC0Args::Cyllinder(cyllinder) => {
                Self::clamp_patches(&mut cyllinder.around_patches);
                Self::clamp_patches(&mut cyllinder.along_patches);
                Self::clamp_length(&mut cyllinder.length);
                Self::clamp_length(&mut cyllinder.radius);
            }
        }
    }

    fn clamp_patches(patches: &mut i32) {
        if *patches < Self::MIN_PATCHES {
            *patches = Self::MIN_PATCHES;
        } else if *patches > Self::MAX_PATCHES {
            *patches = Self::MAX_PATCHES;
        }
    }

    fn clamp_length(length: &mut f32) {
        if *length < Self::MIN_LENGTH {
            *length = Self::MIN_LENGTH;
        } else if *length > Self::MAX_LENGTH {
            *length = Self::MAX_LENGTH;
        }
    }
}

pub struct MainControl<'gl, 'a> {
    entity_manager: &'a RefCell<EntityManager<'gl>>,
    shader_manager: Rc<ShaderManager<'gl>>,
    bezier_surface_args: Option<BezierSurfaceC0Args>,
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
            bezier_surface_args: None,
        }
    }

    pub fn build_ui(&mut self, ui: &mut imgui::Ui, state: &mut State<'gl, '_>) {
        self.main_control_window(ui, state);
        self.selection_window(ui, state);

        if self.bezier_surface_args.is_some() {
            self.bezier_surface_window(ui, state);
        }
    }

    fn main_control_window(&mut self, ui: &imgui::Ui, state: &mut State) {
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

        ui.slider_config("Screen distance", 0.2, 5.0)
            .flags(imgui::SliderFlags::NO_INPUT)
            .build(&mut state.camera.screen_distance);

        if let Some(stereo) = &mut state.camera.stereo {
            ui.slider_config("Baseline", 0.01, 0.50)
                .flags(imgui::SliderFlags::NO_INPUT)
                .build(&mut stereo.baseline);
        }
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
            self.bezier_surface_args = Some(BezierSurfaceC0Args::new_surface());
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

    fn add_bezier_surface_c0(&self, state: &mut State) {}

    fn bezier_surface_window(&mut self, ui: &imgui::Ui, state: &mut State) {
        ui.window("Bezier surface creation")
            .size([350.0, 200.0], imgui::Condition::FirstUseEver)
            .position([300.0, 300.0], imgui::Condition::FirstUseEver)
            .build(|| {
                let args = self.bezier_surface_args.as_mut().unwrap();
                let _token = ui.push_id("bezier_creation_window");

                match args {
                    BezierSurfaceC0Args::Surface(..) => {
                        if ui.button("Surface") {
                            *args = BezierSurfaceC0Args::new_cyllinder();
                        }
                    }
                    BezierSurfaceC0Args::Cyllinder(..) => {
                        if ui.button("Cyllinder") {
                            *args = BezierSurfaceC0Args::new_surface();
                        }
                    }
                }

                match args {
                    BezierSurfaceC0Args::Surface(surface) => {
                        ui.input_int("X patches", &mut surface.x_patches).build();
                        ui.input_int("Z patches", &mut surface.z_patches).build();

                        ui.input_float("X length", &mut surface.x_length).build();
                        ui.input_float("Z length", &mut surface.z_length).build();
                    }
                    BezierSurfaceC0Args::Cyllinder(cyllinder) => {
                        ui.input_int("Around patches", &mut cyllinder.around_patches)
                            .build();
                        ui.input_int("Along patches", &mut cyllinder.along_patches)
                            .build();

                        ui.input_float("Length", &mut cyllinder.length).build();
                        ui.input_float("Radius", &mut cyllinder.radius).build();
                    }
                }

                args.clamp_values();

                ui.columns(2, "bezier_columns", false);
                if ui.button("Ok") {
                    self.add_bezier_surface_c0(state);
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

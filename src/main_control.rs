use crate::{json, state::State, path_gen_ui::path_gen_ui};
use kalimorfia::{
    camera::Stereo,
    entities::{
        basic::{LinearTransformEntity, Translation},
        bezier_surface_args::BezierSurfaceArgs,
        bezier_surface_c0::BezierSurfaceC0,
        bezier_surface_c2::BezierSurfaceC2,
        cnc_block::{CNCBlock, CNCBlockArgs},
        cubic_spline_c0::CubicSplineC0,
        cubic_spline_c2::CubicSplineC2,
        entity::{Entity, EntityCollection, ReferentialSceneEntity, SceneObject},
        gregory_patch::GregoryPatch,
        interpolating_spline::InterpolatingSpline,
        intersection_curve::IntersectionCurve,
        manager::EntityManager,
        point::Point,
        torus::Torus,
    },
    graph::C0EdgeGraph,
    math::{
        geometry::{
            intersection::{Intersection, IntersectionFinder},
            parametric_form::DifferentialParametricForm,
        },
        utils::{point_32_to_64, point_64_to_32},
    },
    render::{shader_manager::ShaderManager, texture::Texture},
    repositories::NameRepository,
    ui::selector::Selector,
};
use nalgebra::{Point3, Vector3};
use std::{cell::RefCell, io::Write, rc::Rc, str::FromStr};

enum BezierSurfaceType {
    C0,
    C2,
}

pub struct MainControl<'gl, 'a> {
    pub entity_manager: &'a RefCell<EntityManager<'gl>>,
    pub shader_manager: Rc<ShaderManager<'gl>>,
    bezier_surface_args: Option<BezierSurfaceArgs>,
    added_surface_type: Option<BezierSurfaceType>,
    cnc_block_args: Option<CNCBlockArgs>,
    intersection_parameters: Option<IntersetionParameters>,
    file_path: String,
    pub gl: &'gl glow::Context,
}

struct IntersectionTarget {
    name: String,
    surface: Box<dyn DifferentialParametricForm<2, 3>>,
    id: usize,
}

struct IntersetionParameters {
    use_cursor: bool,
    numerical_step: f64,
    search_step: f64,
    target_0: IntersectionTarget,
    target_1: IntersectionTarget,
}

const NUMERICAL_STEP_MIN: f64 = 0.001;
const NUMERICAL_STEP_MAX: f64 = 0.01;
const INTERSECTION_STEP_MIN: f64 = 0.001;
const INTERSECTION_STEP_MAX: f64 = 1.0;

impl<'gl, 'a> MainControl<'gl, 'a> {
    pub fn new(
        shader_manager: Rc<ShaderManager<'gl>>,
        entity_manager: &'a RefCell<EntityManager<'gl>>,
        gl: &'gl glow::Context,
    ) -> Self {
        Self {
            file_path: std::env::current_dir()
                .map(|p| String::from(p.to_str().unwrap_or("/")))
                .unwrap_or(String::from("/"))
                + "/file.json",
            added_surface_type: None,
            entity_manager,
            gl,
            intersection_parameters: None,
            shader_manager,
            bezier_surface_args: None,
            cnc_block_args: None,
        }
    }

    pub fn build_ui(&mut self, ui: &mut imgui::Ui, state: &mut State<'gl, 'a>) {
        self.main_control_window(ui, state);
        self.entities_window(ui, state);
        self.selection_window(ui, state);
        path_gen_ui(ui, state, self);

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

        if self.cnc_block_args.is_some() {
            self.cnc_block_window(ui, state);
        }
    }

    fn main_control_window(&mut self, ui: &imgui::Ui, state: &mut State<'gl, 'a>) {
        ui.window("Main control")
            .size([500.0, 400.0], imgui::Condition::FirstUseEver)
            .position([0.0, 0.0], imgui::Condition::FirstUseEver)
            .build(|| {
                ui.separator();
                self.cursor_control(ui, state);
                ui.separator();
                self.display_control(ui, state);
                ui.separator();
                self.file_control(ui, state);
                ui.separator();
                self.additional_control(ui, state);
                ui.separator();
                self.object_creation(ui, state);
            });
    }

    fn entities_window(&mut self, ui: &imgui::Ui, state: &mut State<'gl, 'a>) {
        ui.window("Entities")
            .size([500.0, 300.0], imgui::Condition::FirstUseEver)
            .position([0.0, 400.0], imgui::Condition::FirstUseEver)
            .build(|| {
                ui.separator();
                state.selector.control_ui(ui, self.entity_manager);
            });
    }

    fn selection_window(&self, ui: &imgui::Ui, state: &mut State) {
        let _token = ui.push_id("selection_window");
        ui.window("Selection")
            .size([500.0, 300.0], imgui::Condition::FirstUseEver)
            .position([0.0, 700.0], imgui::Condition::FirstUseEver)
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

    fn display_control(&self, ui: &imgui::Ui, state: &mut State) {
        let _token = ui.push_id("stereoscopy");
        ui.checkbox("GK mode", &mut state.gk_mode);
        let mut stereoscopy = state.camera.stereo.is_some();

        if ui.checkbox("Stereoscopy", &mut stereoscopy) {
            state.camera.stereo = if stereoscopy {
                Some(Stereo::new())
            } else {
                None
            };
        }

        ui.slider_config("Screen distance", 0.2, 50.0)
            .flags(imgui::SliderFlags::NO_INPUT | imgui::SliderFlags::LOGARITHMIC)
            .build(&mut state.camera.screen_distance);

        if let Some(stereo) = &mut state.camera.stereo {
            ui.slider_config("Baseline", 0.01, 0.50)
                .flags(imgui::SliderFlags::NO_INPUT)
                .build(&mut stereo.baseline);
        }
    }

    fn save_scene(&self, state: &State) -> Result<(), ()> {
        let scene_json = json::serialize_scene(&self.entity_manager.borrow(), state).to_string();
        let mut file = std::fs::File::create(&self.file_path).map_err(|_| ())?;
        file.write_all(&scene_json.into_bytes()).map_err(|_| ())?;
        Ok(())
    }

    fn reset_scene(&mut self, state: &mut State<'gl, 'a>) {
        self.entity_manager.borrow_mut().reset();
        self.bezier_surface_args = None;
        self.added_surface_type = None;
        state.reset(
            self.gl,
            self.entity_manager,
            Rc::clone(&self.shader_manager),
        );
    }

    fn load_scene(&mut self, state: &mut State<'gl, 'a>) -> Result<(), ()> {
        self.reset_scene(state);

        let file_contents = std::fs::read_to_string(&self.file_path).map_err(|_| ())?;
        let json = serde_json::Value::from_str(&file_contents).map_err(|_| ())?;
        json::deserialize_scene(
            self.gl,
            &self.shader_manager,
            json,
            &mut self.entity_manager.borrow_mut(),
            state,
        )?;

        Ok(())
    }

    fn file_control(&mut self, ui: &imgui::Ui, state: &mut State<'gl, 'a>) {
        ui.input_text("File path", &mut self.file_path).build();

        ui.columns(2, "file_columns", false);
        if ui.button("Load file") && self.load_scene(state).is_err() {
            self.reset_scene(state);
            ui.open_popup("file_io_error");
        }

        ui.next_column();
        if ui.button("Save to file") && self.save_scene(state).is_err() {
            ui.open_popup("file_io_error");
        }

        ui.popup("file_io_error", || {
            ui.text("Error while performing file IO");
        });
        ui.next_column();
        ui.columns(1, "file_reset_columns", false);
    }

    fn additional_control(&mut self, ui: &imgui::Ui, state: &mut State) {
        ui.columns(3, "additional columns", false);
        self.select_deselect_all(ui, state);
        self.convert_intersection_to_interpolation(ui, state);
        ui.next_column();
        self.remove_selected(ui, state);
        self.merge_points(ui, state);
        ui.next_column();
        self.select_children(ui, state);
        self.generate_intersections(ui, state);
        ui.next_column();
        ui.columns(1, "additional columns clear", false);
    }

    fn select_deselect_all(&self, ui: &imgui::Ui, state: &mut State) {
        if ui.button("Select all") {
            state.selector.select_all();
        }

        if ui.button("Deselect all") {
            state.selector.deselect_all();
        }
    }

    fn merge_points(&self, ui: &imgui::Ui, state: &mut State) {
        if !ui.button("Merge selected points") {
            return;
        }

        let points = state
            .selector
            .selected()
            .iter()
            .copied()
            .filter(|&e| self.entity_manager.borrow().get_entity(e).is_single_point())
            .collect();

        self.entity_manager.borrow_mut().merge_points(points);
        state.selector.deselect_all();
    }

    fn convert_intersection_to_interpolation(&self, ui: &imgui::Ui, state: &mut State) {
        ui.popup("inter_inter_conv_fail", || {
            ui.text("Select exactly one intersection to convert it to an interpolating spline!");
        });

        if !ui.button("Intersect.->interpol.") {
            return;
        }

        let Some(only_selected) = state.selector.only_selected() else {
            ui.open_popup("inter_inter_conv_fail");
            return;
        };

        let manager = self.entity_manager.borrow();
        let selected_entity = manager.get_entity(only_selected);
        let Some(intersection_curve) = selected_entity.as_intersection() else {
            ui.open_popup("inter_inter_conv_fail");
            return;
        };

        let intersection = intersection_curve.intersection().clone();
        std::mem::drop(selected_entity);
        std::mem::drop(manager);

        let point_ids: Vec<_> = intersection
            .points
            .iter()
            .map(|point| self.add_point_at(state, point_64_to_32(point.point)))
            .collect();

        self.add_interpolating_spline_through(state, point_ids, intersection.looped);
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

    fn select_children(&self, ui: &imgui::Ui, state: &mut State) {
        if !ui.button("Select children") {
            return;
        }

        for selected in state.selector.selected() {
            let subscriptions = self
                .entity_manager
                .borrow()
                .subscriptions_of(selected)
                .clone();
            for subscribee in subscriptions {
                state.selector.select(subscribee);
            }
        }
    }

    fn generate_intersections(&mut self, ui: &imgui::Ui, state: &mut State) {
        ui.popup("intersection_selection_error", || {
            ui.text("To generate an intersection, exactly 1 or 2 surface entities are required.");
        });

        if ui.button("Intersections") {
            let Some((target0, target1)) = self.intersection_targets(state) else {
                ui.open_popup("intersection_selection_error");
                return;
            };

            self.intersection_parameters.replace(IntersetionParameters {
                use_cursor: false,
                numerical_step: NUMERICAL_STEP_MIN * 5.0,
                search_step: INTERSECTION_STEP_MIN * 10.0,
                target_0: target0,
                target_1: target1,
            });

            ui.open_popup("intersection_window")
        }

        if self.intersection_parameters.is_some() {
            ui.popup("intersection_window", || {
                ui.popup("intersection_not_found", || {
                    ui.text("No intersection found between selected surfaces");
                });

                let target0 = &self.intersection_parameters.as_ref().unwrap().target_0;
                let target1 = &self.intersection_parameters.as_ref().unwrap().target_1;

                let self_intersection = target0.id == target1.id;

                if self_intersection {
                    ui.text(format!("Intersecting {} with itself", target0.name));
                } else {
                    ui.text(format!(
                        "Intersecting {} and {}",
                        target0.name, target1.name
                    ));
                }

                let params = self.intersection_parameters.as_mut().unwrap();

                ui.slider_config("Numerical step", NUMERICAL_STEP_MIN, NUMERICAL_STEP_MAX)
                    .flags(imgui::SliderFlags::LOGARITHMIC)
                    .build(&mut params.numerical_step);

                params.numerical_step = params
                    .numerical_step
                    .clamp(NUMERICAL_STEP_MIN, NUMERICAL_STEP_MAX);

                ui.slider_config(
                    "Intersection step",
                    INTERSECTION_STEP_MIN,
                    INTERSECTION_STEP_MAX,
                )
                .flags(imgui::SliderFlags::LOGARITHMIC)
                .build(&mut params.search_step);

                params.search_step = params
                    .search_step
                    .clamp(INTERSECTION_STEP_MIN, INTERSECTION_STEP_MAX);

                ui.checkbox("Use cursor as starting point", &mut params.use_cursor);

                ui.columns(2, "Intersection columns", false);
                if ui.button("Ok") {
                    let guide = params
                        .use_cursor
                        .then_some(state.cursor.location())
                        .flatten()
                        .map(point_32_to_64);

                    let mut intersection_finder = if self_intersection {
                        IntersectionFinder::new_same(&*params.target_0.surface)
                    } else {
                        IntersectionFinder::new(
                            &*params.target_0.surface,
                            &*params.target_1.surface,
                        )
                    };

                    intersection_finder.guide_point = guide;
                    intersection_finder.numerical_step = params.numerical_step;
                    intersection_finder.intersection_step = params.search_step;

                    let intersection = intersection_finder.find();

                    if let Some(intersection) = intersection {
                        let [texture_0, texture_1] = Texture::intersection_texture(
                            &intersection,
                            &*params.target_0.surface,
                            &*params.target_1.surface,
                            1000,
                        );

                        self.entity_manager
                            .borrow_mut()
                            .get_entity_mut(params.target_0.id)
                            .set_intersection_texture(texture_0);

                        self.entity_manager
                            .borrow_mut()
                            .get_entity_mut(params.target_1.id)
                            .set_intersection_texture(texture_1);

                        self.add_intersection_curve(state, intersection);

                        self.intersection_parameters = None;
                    } else {
                        ui.open_popup("intersection_not_found");
                    }
                }

                ui.next_column();
                if ui.button("Cancel") {
                    self.intersection_parameters = None;
                }

                ui.next_column();
                ui.columns(1, "clear_columns_intersect", false);
            });
        }
    }

    fn intersection_targets(
        &self,
        state: &State,
    ) -> Option<(IntersectionTarget, IntersectionTarget)> {
        let manager = self.entity_manager.borrow();

        let targets: Vec<_> = state
            .selector
            .selected()
            .iter()
            .copied()
            .filter(|&id| manager.get_entity(id).as_parametric_2_to_3().is_some())
            .collect();

        if targets.len() == 2 {
            let target0 = manager.get_entity(targets[0]);
            let target1 = manager.get_entity(targets[1]);
            Some((
                IntersectionTarget {
                    name: target0.name(),
                    surface: target0.as_parametric_2_to_3().unwrap(),
                    id: targets[0],
                },
                IntersectionTarget {
                    name: target1.name(),
                    surface: target1.as_parametric_2_to_3().unwrap(),
                    id: targets[1],
                },
            ))
        } else if targets.len() == 1 {
            let target = manager.get_entity(targets[0]);
            Some((
                IntersectionTarget {
                    name: target.name(),
                    surface: target.as_parametric_2_to_3().unwrap(),
                    id: targets[0],
                },
                IntersectionTarget {
                    name: target.name(),
                    surface: target.as_parametric_2_to_3().unwrap(),
                    id: targets[0],
                },
            ))
        } else {
            None
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
            self.bezier_surface_args = Some(BezierSurfaceArgs::new_surface());
            self.added_surface_type = Some(BezierSurfaceType::C0);
        }

        ui.next_column();
        if ui.button("Bezier surface C2") {
            self.bezier_surface_args = Some(BezierSurfaceArgs::new_surface());
            self.added_surface_type = Some(BezierSurfaceType::C2);
        }

        ui.next_column();
        if ui.button("Gregory patch") {
            self.add_gregory_patch(state);
        }

        ui.next_column();
        if ui.button("CNC block") {
            self.cnc_block_args = Some(CNCBlockArgs::new());
        }

        ui.next_column();
        ui.columns(1, "clear_columns", false);
    }

    fn cnc_block_window(&mut self, ui: &imgui::Ui, state: &mut State) {
        ui.window("CNC block creation")
            .size([350.0, 250.0], imgui::Condition::FirstUseEver)
            .position([300.0, 300.0], imgui::Condition::FirstUseEver)
            .build(|| {
                let args = self.cnc_block_args.as_mut().unwrap();

                ui.text("CNC block creation");
                ui.input_int("Samples X", &mut args.sampling.x).build();
                ui.input_int("Samples Y", &mut args.sampling.y).build();

                ui.input_float("Size X", &mut args.size.x).build();
                ui.input_float("Size Y", &mut args.size.y).build();
                ui.input_float("Size Z", &mut args.size.z).build();

                args.clamp();

                if ui.button("Create") {
                    let args = self.cnc_block_args.take().unwrap();
                    self.add_cnc_block(state, args);
                    return;
                }

                if ui.button("Cancel") {
                    self.cnc_block_args = None;
                }
            });
    }

    fn add_point_at(&self, state: &mut State, position: Point3<f32>) -> usize {
        let point = Box::new(Point::with_position(
            self.gl,
            position,
            Rc::clone(&state.name_repo),
            Rc::clone(&self.shader_manager),
        ));

        let id = self.entity_manager.borrow_mut().add_entity(point);
        state.selector.add_selectable(id);
        id
    }

    /// Adds a point without adding it to the only selected entity if such an entity exists
    fn quietly_add_point(&self, state: &mut State) -> usize {
        self.add_point_at(state, state.cursor.location().unwrap())
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
            selected_points.clone(),
            self.entity_manager.borrow().entities(),
        );

        self.add_spline(state, spline, &selected_points);
    }

    fn add_cubic_spline_c2(&self, state: &mut State) {
        let selected_points = self.selected_points(&state.selector);
        let spline = CubicSplineC2::through_points(
            self.gl,
            Rc::clone(&state.name_repo),
            Rc::clone(&self.shader_manager),
            selected_points.clone(),
            self.entity_manager.borrow().entities(),
        );

        self.add_spline(state, spline, &selected_points);
    }

    fn add_interpolating_spline_through(
        &self,
        state: &mut State,
        point_ids: Vec<usize>,
        looped: bool,
    ) {
        let mut spline = InterpolatingSpline::through_points(
            self.gl,
            Rc::clone(&state.name_repo),
            Rc::clone(&self.shader_manager),
            point_ids.clone(),
            self.entity_manager.borrow().entities(),
        );

        spline.looped = looped;
        self.add_spline(state, spline, &point_ids);
    }

    fn add_interpolating_spline(&self, state: &mut State) {
        let selected_points = self.selected_points(&state.selector);
        self.add_interpolating_spline_through(state, selected_points, false);
    }

    fn add_spline<T: ReferentialSceneEntity<'gl> + 'gl>(
        &self,
        state: &mut State,
        spline: T,
        points: &[usize],
    ) {
        let boxed_spline = Box::new(spline);

        let id = self.entity_manager.borrow_mut().add_entity(boxed_spline);

        for &point in points {
            self.entity_manager.borrow_mut().subscribe(id, point);
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

    fn add_gregory_patch(&self, state: &mut State) {
        let entity_manager = self.entity_manager.borrow();

        let selected_surface_ids: Vec<_> = state
            .selector
            .selected()
            .iter()
            .filter_map(|&id| entity_manager.get_entity(id).as_c0_surface().and(Some(id)))
            .collect();

        let triangles = C0EdgeGraph::new(
            self.entity_manager.borrow().entities(),
            &selected_surface_ids,
        )
        .find_triangles();

        std::mem::drop(entity_manager);

        for triangle in triangles {
            let gregory = Box::new(GregoryPatch::new(
                self.gl,
                Rc::clone(&state.name_repo),
                Rc::clone(&self.shader_manager),
                self.entity_manager.borrow().entities(),
                triangle.clone(),
            ));

            let id = self.entity_manager.borrow_mut().add_entity(gregory);
            state.selector.add_selectable(id);

            for edge in triangle.0 {
                for &point in edge.points.iter().flatten() {
                    self.entity_manager.borrow_mut().subscribe(id, point);
                }
            }
        }
    }

    pub fn add_cnc_block(&self, state: &mut State, args: CNCBlockArgs) {
        let block = Box::new(CNCBlock::new(
            self.gl,
            Rc::clone(&state.name_repo),
            Rc::clone(&self.shader_manager),
            args,
        ));

        let id = self.entity_manager.borrow_mut().add_entity(block);
        state.selector.add_selectable(id);
    }

    pub fn add_intersection_curve(&self, state: &mut State, intersection: Intersection) {
        let intersection_curve = Box::new(IntersectionCurve::new(
            self.gl,
            Rc::clone(&state.name_repo),
            Rc::clone(&self.shader_manager),
            intersection,
        ));

        let id = self
            .entity_manager
            .borrow_mut()
            .add_entity(intersection_curve);
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

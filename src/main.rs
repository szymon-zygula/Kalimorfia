use glow::HasContext;
use glutin::platform::run_return::EventLoopExtRunReturn;
use kalimorfia::{
    camera::Camera,
    entities::{
        aggregate::Aggregate,
        cubic_spline_c0::CubicSplineC0,
        cursor::ScreenCursor,
        entity::{DrawType, Drawable, Entity, ReferentialSceneEntity, SceneObject},
        manager::EntityManager,
        point::Point,
        scene_grid::SceneGrid,
        torus::Torus,
    },
    mouse::MouseState,
    primitives::color::ColorAlpha,
    repositories::{ExactNameRepository, NameRepository, UniqueNameRepository},
    ui::selector::Selector,
    window::Window,
};
use nalgebra::{Matrix4, Point2};
use std::{cell::RefCell, rc::Rc, time::Instant};

const WINDOW_TITLE: &str = "Kalimorfia";
const WINDOW_WIDTH: u32 = 1280;
const WINDOW_HEIGHT: u32 = 720;
const CLEAR_COLOR: ColorAlpha = ColorAlpha {
    r: 0.4,
    g: 0.4,
    b: 0.4,
    a: 1.0,
};

struct State<'gl, 'a> {
    pub cursor: ScreenCursor<'gl>,
    pub camera: Camera,
    pub selector: Selector<'a>,
    pub name_repo: Rc<RefCell<dyn NameRepository>>,
    pub selected_aggregate_id: usize,
}

impl<'gl, 'a> State<'gl, 'a> {
    pub fn add_entity(
        &mut self,
        entity: Box<dyn ReferentialSceneEntity<'gl> + 'gl>,
        entity_manager: &mut EntityManager<'gl>,
    ) {
        let id = entity_manager.add_entity(entity);
        self.selector.add_selectable(id);
    }
}

fn build_ui<'gl>(
    gl: &'gl glow::Context,
    ui: &mut imgui::Ui,
    state: &mut State<'gl, '_>,
    entity_manager: &RefCell<EntityManager<'gl>>,
) {
    ui.window("Main control")
        .size([500.0, 300.0], imgui::Condition::FirstUseEver)
        .position([0.0, 0.0], imgui::Condition::FirstUseEver)
        .build(|| {
            ui.separator();
            state.cursor.control_ui(ui);

            if ui.button("Center on cursor") {
                state.camera.center = state.cursor.location().unwrap();
            }

            ui.separator();
            ui.text("Object creation");
            ui.columns(3, "creation_columns", false);
            if ui.button("Torus") {
                let id = entity_manager
                    .borrow_mut()
                    .add_entity(Box::new(Torus::with_position(
                        gl,
                        state.cursor.location().unwrap(),
                        Rc::clone(&state.name_repo),
                    )));
                state.selector.add_selectable(id);
            };

            ui.next_column();
            if ui.button("Point") {
                let point = Box::new(Point::with_position(
                    gl,
                    state.cursor.location().unwrap(),
                    Rc::clone(&state.name_repo),
                ));

                let id = entity_manager.borrow_mut().add_entity(point);
                state.selector.add_selectable(id);

                if let Some(only_id) = state.selector.only_selected() {
                    if entity_manager.borrow().entities()[&only_id]
                        .borrow_mut()
                        .add_point(id, entity_manager.borrow().entities())
                    {
                        entity_manager.borrow_mut().subscribe(only_id, id);
                    }
                }
            }

            ui.next_column();
            if ui.button("Cubic Spline C0") {
                let mut selected: Vec<usize> = state
                    .selector
                    .selected()
                    .iter()
                    .filter(|&&id| entity_manager.borrow().get_entity(id).is_single_point())
                    .copied()
                    .collect();
                selected.sort();
                let spline = Box::new(CubicSplineC0::through_points(
                    gl,
                    Rc::clone(&state.name_repo),
                    selected.clone(),
                    entity_manager,
                ));

                let id = entity_manager.borrow_mut().add_entity(spline);

                for selected in selected {
                    entity_manager.borrow_mut().subscribe(id, selected);
                }

                state.selector.add_selectable(id);
            }

            ui.next_column();

            ui.separator();

            state.selector.control_ui(ui, entity_manager);
        });

    ui.window("Selection")
        .size([500.0, 500.0], imgui::Condition::FirstUseEver)
        .position([0.0, 300.0], imgui::Condition::FirstUseEver)
        .build(|| {
            ui.separator();
            entity_manager
                .borrow_mut()
                .control_referential_ui(state.selected_aggregate_id, ui);
        });
}

fn select_clicked(
    pixel: glutin::dpi::PhysicalPosition<f64>,
    state: &mut State,
    resolution: &glutin::dpi::PhysicalSize<u32>,
    entity_manager: &RefCell<EntityManager>,
) {
    let point = Point2::new(
        2.0 * (pixel.x as f32 + 0.5) / resolution.width as f32 - 1.0,
        -(2.0 * (pixel.y as f32 + 0.5) / resolution.height as f32 - 1.0),
    );

    let mut closest_id = None;
    let mut closest_dist = f32::INFINITY;

    for (&id, entity) in entity_manager.borrow().entities() {
        let (is_at_point, camera_distance) = entity.borrow().is_at_point(
            point,
            &state.camera.projection_transform(),
            &state.camera.view_transform(),
            resolution,
        );

        if is_at_point && camera_distance < closest_dist {
            closest_dist = camera_distance;
            closest_id = Some(id);
        }
    }

    if let Some(id) = closest_id {
        state.selector.toggle(id);
    }
}

fn main() {
    let (mut window, mut event_loop, gl) = Window::new(WINDOW_TITLE, WINDOW_WIDTH, WINDOW_HEIGHT);
    let mut last_frame = Instant::now();
    let mut mouse = MouseState::new();
    let grid = SceneGrid::new(&gl, 100, 50.0);

    let entity_manager = RefCell::new(EntityManager::new());
    let selected_aggregate_id = entity_manager
        .borrow_mut()
        .add_entity(Box::new(Aggregate::new(
            &gl,
            &mut ExactNameRepository::new(),
        )));

    let mut state = State {
        camera: Camera::new(),
        cursor: ScreenCursor::new(&gl, Camera::new(), window.size()),
        name_repo: Rc::new(RefCell::new(UniqueNameRepository::new())),
        selector: Selector::new(
            |id| {
                entity_manager
                    .borrow_mut()
                    .subscribe(selected_aggregate_id, id);
            },
            |id| {
                entity_manager
                    .borrow_mut()
                    .unsubscribe(selected_aggregate_id, id);
            },
            |id| {
                entity_manager.borrow_mut().remove_entity(id);
            },
        ),
        selected_aggregate_id,
    };

    unsafe {
        gl.clear_color(CLEAR_COLOR.r, CLEAR_COLOR.g, CLEAR_COLOR.b, CLEAR_COLOR.a);
    }

    use glutin::event::{Event, WindowEvent};

    event_loop.run_return(|event, _, control_flow| match event {
        Event::NewEvents(_) => {
            let now = Instant::now();
            let duration = now.duration_since(last_frame);
            window.update_delta_time(duration);
            last_frame = now;
        }
        Event::MainEventsCleared => window.request_redraw(),
        Event::RedrawRequested(_) => {
            unsafe {
                gl.clear(glow::COLOR_BUFFER_BIT);
            }

            if state.camera.update_from_mouse(&mut mouse, &window) {
                state.cursor.set_camera(&state.camera);
            }

            if mouse.has_left_button_been_pressed() && !window.imgui_using_mouse() {
                if let Some(position) = mouse.position() {
                    select_clicked(position, &mut state, &window.size(), &entity_manager);
                }
            }

            grid.draw_normal(&state.camera);

            for id in state.selector.selectables() {
                entity_manager.borrow().draw_referential(
                    id,
                    &state.camera,
                    &Matrix4::identity(),
                    DrawType::Selected,
                );
            }

            entity_manager.borrow().draw_referential(
                selected_aggregate_id,
                &state.camera,
                &Matrix4::identity(),
                DrawType::Normal,
            );

            state.cursor.draw_normal(&state.camera);
            window.render(&gl, |ui| build_ui(&gl, ui, &mut state, &entity_manager));
        }
        Event::WindowEvent {
            event: WindowEvent::CloseRequested,
            ..
        } => *control_flow = glutin::event_loop::ControlFlow::Exit,
        event => {
            if let Event::WindowEvent { ref event, .. } = event {
                mouse.handle_window_event(event);

                if let WindowEvent::Resized(resolution) = event {
                    state.camera.window_size = resolution.clone();
                    state
                        .cursor
                        .set_camera_and_resolution(&state.camera, resolution);
                }
            }

            window.handle_event(event, &gl);
        }
    });

    drop(state);
}

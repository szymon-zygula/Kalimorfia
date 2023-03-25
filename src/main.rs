use glow::HasContext;
use glutin::platform::run_return::EventLoopExtRunReturn;
use kalimorfia::{
    camera::Camera,
    entities::{
        aggregate::Aggregate,
        cursor::ScreenCursor,
        entity::{Drawable, Entity, ReferentialSceneEntity, SceneObject},
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
use nalgebra::Point2;
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

struct State<'gl, S: FnMut(usize), D: FnMut(usize), R: FnMut(usize)> {
    pub cursor: ScreenCursor<'gl>,
    pub camera: Camera,
    pub selector: Selector<S, D, R>,
    pub name_repo: Rc<RefCell<dyn NameRepository>>,
    pub selected_aggregate_id: usize,
}

impl<'gl, S: FnMut(usize), D: FnMut(usize), R: FnMut(usize)> State<'gl, S, D, R> {
    pub fn add_entity(
        &mut self,
        entity: Box<dyn ReferentialSceneEntity<'gl> + 'gl>,
        entity_manager: &mut EntityManager<'gl>,
    ) {
        let id = entity_manager.add_entity(entity);
        self.selector.add_selectable(id);
    }
}

fn build_ui<'gl, S: FnMut(usize), D: FnMut(usize), R: FnMut(usize)>(
    gl: &'gl glow::Context,
    ui: &mut imgui::Ui,
    state: &mut State<'gl, S, D, R>,
    entity_manager: &RefCell<EntityManager<'gl>>,
) {
    ui.window("Main control")
        .size([500.0, 300.0], imgui::Condition::FirstUseEver)
        .position([0.0, 0.0], imgui::Condition::FirstUseEver)
        .build(|| {
            ui.separator();
            state.cursor.control_ui(ui);

            if ui.button("Center on cursor") {
                state.camera.center = state.cursor.location();
            }

            ui.separator();
            ui.text("Object creation");
            if ui.button("Torus") {
                let id = entity_manager
                    .borrow_mut()
                    .add_entity(Box::new(Torus::with_position(
                        gl,
                        state.cursor.location(),
                        Rc::clone(&state.name_repo),
                    )));
                state.selector.add_selectable(id);
            }

            if ui.button("Point") {
                let id = entity_manager
                    .borrow_mut()
                    .add_entity(Box::new(Point::with_position(
                        gl,
                        state.cursor.location(),
                        Rc::clone(&state.name_repo),
                    )));
                state.selector.add_selectable(id);
            }

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

fn select_clicked<S: FnMut(usize), D: FnMut(usize), R: FnMut(usize)>(
    pixel: glutin::dpi::PhysicalPosition<f64>,
    state: &mut State<S, D, R>,
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

            let view_transform = state.camera.view_transform();
            let projection_transform = state.camera.projection_transform();

            grid.draw(&projection_transform, &view_transform);

            for id in state.selector.unselected() {
                entity_manager.borrow().draw_referential(
                    id,
                    &projection_transform,
                    &view_transform,
                );
            }

            entity_manager.borrow().draw_referential(
                selected_aggregate_id,
                &projection_transform,
                &view_transform,
            );

            state.cursor.draw(&projection_transform, &view_transform);
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
                    state.camera.aspect_ratio = resolution.width as f32 / resolution.height as f32;
                    state
                        .cursor
                        .set_camera_and_resolution(&state.camera, resolution);
                }
            }

            window.handle_event(event, &gl);
        }
    });
}

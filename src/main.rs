use glow::HasContext;
use glutin::platform::run_return::EventLoopExtRunReturn;
use kalimorfia::{
    camera::Camera,
    entities::{
        aggregate::Aggregate,
        cursor::Cursor,
        entity::{Entity, SceneEntity, SceneObject},
        point::Point,
        scene_grid::SceneGrid,
        torus::Torus,
    },
    mouse::MouseState,
    primitives::color::ColorAlpha,
    scene::SceneEntityEntry,
    window::Window,
};
use nalgebra::Point2;
use std::collections::BTreeMap;
use std::time::Instant;

const WINDOW_TITLE: &str = "Kalimorfia";
const WINDOW_WIDTH: u32 = 1280;
const WINDOW_HEIGHT: u32 = 720;
const CLEAR_COLOR: ColorAlpha = ColorAlpha {
    r: 0.4,
    g: 0.4,
    b: 0.4,
    a: 1.0,
};

struct State<'gl> {
    pub cursor: Cursor<'gl>,
    pub entries: BTreeMap<usize, SceneEntityEntry<'gl>>,
    pub selected_aggregate: Aggregate<'gl>,
    pub camera: Camera,
    next_id: usize,
}

impl<'gl> State<'gl> {
    pub fn add_entity(&mut self, entity: Box<dyn SceneEntity + 'gl>) {
        let (name, id) = self.get_next_entity_name_and_id();
        self.entries.insert(
            id,
            SceneEntityEntry {
                name: name.clone(),
                new_name: name,
                entity: Some(entity),
                id,
            },
        );
    }

    pub fn toggle_entry(&mut self, id: usize) {
        let entry = self.entries.get_mut(&id).unwrap();
        match entry.entity.take() {
            Some(taken_object) => self.selected_aggregate.add_object(id, taken_object),
            None => entry.entity = Some(self.selected_aggregate.take_object(id)),
        }
    }

    pub fn remove_entry(&mut self, id: usize) {
        if self.entries.remove(&id).unwrap().entity.is_none() {
            self.selected_aggregate.take_object(id);
        }
    }

    pub fn entity(&self, id: usize) -> &dyn SceneEntity {
        match self.entries[&id].entity {
            Some(ref entity) => entity.as_ref(),
            None => self.selected_aggregate.get_entity(id),
        }
    }

    fn get_next_entity_name_and_id(&mut self) -> (String, usize) {
        let name = format!("Entity {}", self.next_id);
        let id = self.next_id;
        self.next_id += 1;
        (name, id)
    }
}

fn build_ui<'gl>(gl: &'gl glow::Context, ui: &mut imgui::Ui, state: &mut State<'gl>) {
    ui.window("Main control")
        .size([500.0, 300.0], imgui::Condition::FirstUseEver)
        .position([0.0, 0.0], imgui::Condition::FirstUseEver)
        .build(|| {
            ui.separator();
            ui.text("Cursor control");
            state.cursor.control_ui(ui);

            if ui.button("Center on cursor") {
                state.camera.center = state.cursor.position();
            }

            ui.separator();
            ui.text("Object creation");
            if ui.button("Torus") {
                state.add_entity(Box::new(Torus::with_position(gl, state.cursor.position())));
            }

            if ui.button("Point") {
                state.add_entity(Box::new(Point::with_position(gl, state.cursor.position())));
            }

            ui.separator();
            ui.text("Object list");

            for i in Iterator::collect::<Vec<usize>>(state.entries.keys().copied()) {
                let _id = ui.push_id(format!("entry_{}", state.entries[&i].name));

                ui.columns(2, "columns", false);
                let clicked = ui
                    .selectable_config(&state.entries[&i].name)
                    .selected(state.entries[&i].entity.is_none())
                    .build();

                if clicked {
                    state.toggle_entry(i);
                }

                ui.next_column();
                if ui.button_with_size("X", [18.0, 18.0]) {
                    state.remove_entry(i);
                }
                ui.next_column();
            }
        });

    ui.window("Selection")
        .size([500.0, 500.0], imgui::Condition::FirstUseEver)
        .position([0.0, 300.0], imgui::Condition::FirstUseEver)
        .build(|| {
            if state.selected_aggregate.len() == 1 {
                let (id, _) = state.selected_aggregate.only_one();
                ui.input_text("Name", &mut state.entries.get_mut(&id).unwrap().new_name)
                    .build();

                if ui.button("Rename") {
                    let name_taken = state
                        .entries
                        .values()
                        .filter(|x| x.name == state.entries[&id].new_name)
                        .count()
                        != 0;
                    if name_taken && state.entries[&id].new_name != state.entries[&id].name {
                        println!("taken");
                        ui.open_popup("name_taken_popup");
                    } else {
                        state.entries.get_mut(&id).unwrap().name =
                            state.entries[&id].new_name.clone();
                    }
                }

                ui.popup("name_taken_popup", || {
                    ui.text("Name already taken");
                });
            }

            ui.separator();
            state.selected_aggregate.control_ui(ui);
        });
}

fn select_clicked(
    pixel: glutin::dpi::PhysicalPosition<f64>,
    state: &mut State,
    resolution: &glutin::dpi::PhysicalSize<u32>,
) {
    let point = Point2::new(
        2.0 * (pixel.x as f32 + 0.5) / resolution.width as f32 - 1.0,
        -(2.0 * (pixel.y as f32 + 0.5) / resolution.height as f32 - 1.0),
    );

    let mut closest_idx = None;
    let mut closest_dist = f32::INFINITY;

    for &id in state.entries.keys() {
        let (is_at_point, camera_distance) = state.entity(id).is_at_point(
            point,
            &state.camera.projection_transform(),
            &state.camera.view_transform(),
            resolution,
        );

        if is_at_point && camera_distance < closest_dist {
            closest_dist = camera_distance;
            closest_idx = Some(id);
        }
    }

    if let Some(idx) = closest_idx {
        state.toggle_entry(idx);
    }
}

fn main() {
    let (mut window, mut event_loop, gl) = Window::new(WINDOW_TITLE, WINDOW_WIDTH, WINDOW_HEIGHT);
    let mut last_frame = Instant::now();
    let mut mouse = MouseState::new();
    let grid = SceneGrid::new(&gl, 100, 50.0);
    let mut state = State {
        camera: Camera::new(),
        cursor: Cursor::new(&gl, 1.0),
        entries: BTreeMap::new(),
        selected_aggregate: Aggregate::new(&gl),
        next_id: 0,
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

            state.camera.update_from_mouse(&mut mouse, &window);

            if mouse.has_left_button_been_pressed() && !window.imgui_using_mouse() {
                if let Some(position) = mouse.position() {
                    select_clicked(position, &mut state, &window.size());
                }
            }

            let view_transform = state.camera.view_transform();
            let projection_transform = state.camera.projection_transform();

            grid.draw(&projection_transform, &view_transform);
            for entry in state.entries.values() {
                if let Some(ref entity) = entry.entity {
                    entity.draw(&projection_transform, &view_transform);
                }
            }

            state
                .selected_aggregate
                .draw(&projection_transform, &view_transform);
            state.cursor.draw(&projection_transform, &view_transform);
            window.render(&gl, |ui| build_ui(&gl, ui, &mut state));
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
                }
            }

            window.handle_event(event, &gl);
        }
    });
}

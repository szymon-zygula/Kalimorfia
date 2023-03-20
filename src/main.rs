use glow::HasContext;
use glutin::platform::run_return::EventLoopExtRunReturn;
use kalimorfia::{
    camera::Camera,
    entities::{
        cursor::Cursor,
        entity::{Entity, SceneEntity, SceneObject},
        point::Point,
        scene_grid::SceneGrid,
        torus::Torus,
    },
    mouse::MouseState,
    primitives::color::ColorAlpha,
    window::Window,
};
use nalgebra::Point2;
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

fn next_name(scene_entity_count: usize) -> String {
    format!("Entity {}", scene_entity_count)
}

struct SceneEntityEntry<'gl> {
    pub entity: Box<dyn SceneEntity + 'gl>,
    pub selected: bool,
    pub name: String,
    pub new_name: String,
}

struct State<'gl> {
    pub cursor: Cursor<'gl>,
    pub entries: Vec<SceneEntityEntry<'gl>>,
    pub camera: Camera,
}

impl<'gl> State<'gl> {
    pub fn add_entity(&mut self, entity: Box<dyn SceneEntity + 'gl>) {
        let name = next_name(self.entries.len());
        self.entries.push(SceneEntityEntry {
            name: name.clone(),
            new_name: name,
            entity,
            selected: false,
        });
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

            for entry in &mut state.entries {
                let clicked = ui
                    .selectable_config(&entry.name)
                    .selected(entry.selected)
                    .build();

                if clicked {
                    entry.selected = !entry.selected;
                }
            }
        });

    let mut selected = state.entries.iter().enumerate().filter(|(_, x)| x.selected);
    let unique_idx = if let Some((idx, _)) = selected.next().xor(selected.next()) {
        Some(idx)
    } else {
        None
    };

    if let Some(idx) = unique_idx {
        ui.window("Selected object")
            .size([500.0, 500.0], imgui::Condition::FirstUseEver)
            .position([0.0, 300.0], imgui::Condition::FirstUseEver)
            .build(|| {
                if ui.button("Remove entity") {
                    state.entries.remove(idx);
                } else {
                    ui.input_text("Name", &mut state.entries[idx].new_name)
                        .build();

                    if ui.button("Rename") {
                        let name_taken = state
                            .entries
                            .iter()
                            .filter(|x| x.name == state.entries[idx].new_name)
                            .count()
                            != 0;
                        if name_taken && state.entries[idx].new_name != state.entries[idx].name {
                            println!("taken");
                            ui.open_popup("name_taken_popup");
                        } else {
                            state.entries[idx].name = state.entries[idx].new_name.clone();
                        }
                    }

                    ui.separator();
                    state.entries[idx].entity.control_ui(ui);

                    ui.popup("name_taken_popup", || {
                        ui.text("Name already taken");
                    });
                }
            });
    }
}

fn select_clicked(
    pixel: glutin::dpi::PhysicalPosition<f64>,
    camera: &Camera,
    entries: &mut [SceneEntityEntry],
    resolution: &glutin::dpi::PhysicalSize<u32>,
) {
    let point = Point2::new(
        2.0 * (pixel.x as f32 + 0.5) / resolution.width as f32 - 1.0,
        -(2.0 * (pixel.y as f32 + 0.5) / resolution.height as f32 - 1.0),
    );

    let mut closest_idx = None;
    let mut closest_dist = f32::INFINITY;

    for (idx, entry) in entries.iter().enumerate() {
        let (is_at_point, camera_distance) = entry.entity.is_at_point(
            point,
            &camera.projection_transform(),
            &camera.view_transform(),
            resolution,
        );

        if is_at_point && camera_distance < closest_dist {
            closest_dist = camera_distance;
            closest_idx = Some(idx);
        }
    }

    if let Some(idx) = closest_idx {
        entries[idx].selected = !entries[idx].selected;
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
        entries: Vec::new(),
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
                    select_clicked(position, &state.camera, &mut state.entries, &window.size());
                }
            }

            let view_transform = state.camera.view_transform();
            let projection_transform = state.camera.projection_transform();

            grid.draw(&projection_transform, &view_transform);
            for entry in &state.entries {
                entry.entity.draw(&projection_transform, &view_transform);
            }

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

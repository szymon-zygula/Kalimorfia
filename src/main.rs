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
    math::affine::transforms,
    mouse::MouseState,
    primitives::color::ColorAlpha,
    window::Window,
};
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

fn main() {
    let (mut window, mut event_loop, gl) = Window::new(WINDOW_TITLE, WINDOW_WIDTH, WINDOW_HEIGHT);
    let mut last_frame = Instant::now();
    let mut resolution = glutin::dpi::PhysicalSize::new(WINDOW_WIDTH, WINDOW_HEIGHT);
    let mut mouse = MouseState::new();
    let grid = SceneGrid::new(&gl, 80, 40.0);
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

            let view_transform = state.camera.view_transform();

            let projection_transform = transforms::projection(
                std::f32::consts::FRAC_PI_2,
                resolution.width as f32 / resolution.height as f32,
                0.1,
                100.0,
            );

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

                if let WindowEvent::Resized(size) = event {
                    resolution = *size;
                }
            }

            window.handle_event(event, &gl);
        }
    });
}

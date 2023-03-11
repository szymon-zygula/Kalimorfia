use kalimorfia::{
    math::affine,
    primitives::color::Color,
    render::{gl_program::GlProgram, shader::Shader},
    window::Window,
};

use std::{path::Path, time::Instant};

use glow::HasContext;

const WINDOW_TITLE: &str = "ProForma";
const WINDOW_WIDTH: u32 = 1280;
const WINDOW_HEIGHT: u32 = 720;
const CLEAR_COLOR: Color = Color {
    r: 0.4,
    g: 0.4,
    b: 0.4,
    a: 1.0,
};

#[derive(Debug)]
struct State {
    pub left_mouse_button_down: bool,
    pub right_mouse_button_down: bool,
    pub current_mouse_position: Option<glutin::dpi::PhysicalPosition<f64>>,
    pub previous_mouse_position: Option<glutin::dpi::PhysicalPosition<f64>>,
    pub scroll_delta: f32,
    pub resolution: glutin::dpi::PhysicalSize<u32>,
}

fn build_ui(ui: &mut imgui::Ui, state: &mut State) {
    ui.window("ProForma")
        .size([500.0, 300.0], imgui::Condition::FirstUseEver)
        .position([0.0, 0.0], imgui::Condition::FirstUseEver)
        .build(|| {
            ui.separator();
            ui.text("Imgui works");
        });
}

fn main() {
    let (mut window, event_loop) = Window::new(WINDOW_TITLE, WINDOW_WIDTH, WINDOW_HEIGHT);
    let mut last_frame = Instant::now();

    let mut app_state = State {
        left_mouse_button_down: false,
        right_mouse_button_down: false,
        current_mouse_position: None,
        previous_mouse_position: None,
        scroll_delta: 0.0,
        resolution: glutin::dpi::PhysicalSize::new(WINDOW_WIDTH, WINDOW_HEIGHT),
    };

    let gl = window.gl();

    let vertex_array = unsafe { gl.create_vertex_array() }.unwrap();

    let mut gl_program = Some(GlProgram::with_shader_paths(
        gl,
        vec![
            (Path::new("shaders/simple_vertex.glsl"), glow::VERTEX_SHADER),
            (
                Path::new("shaders/simple_fragment.glsl"),
                glow::FRAGMENT_SHADER,
            ),
        ],
    ));

    window.set_clear_color(CLEAR_COLOR);

    use glutin::event::{Event, WindowEvent};

    event_loop.run(move |event, _, control_flow| match event {
        Event::NewEvents(_) => {
            let now = Instant::now();
            let duration = now.duration_since(last_frame);
            window.update_delta_time(duration);
            last_frame = now;
        }
        Event::MainEventsCleared => window.request_redraw(),
        Event::RedrawRequested(_) => {
            let gl = window.gl();

            unsafe {
                gl.clear(glow::COLOR_BUFFER_BIT);
                gl_program.as_ref().unwrap().use_by(gl);

                gl.bind_vertex_array(Some(vertex_array));
                gl.draw_arrays(glow::TRIANGLES, 0, 3);
            }

            window.render(|ui| build_ui(ui, &mut app_state));
        }
        Event::WindowEvent {
            event:
                WindowEvent::MouseWheel {
                    delta: glutin::event::MouseScrollDelta::LineDelta(_, delta),
                    ..
                },
            ..
        } => {
            app_state.scroll_delta = delta;
        }
        Event::WindowEvent {
            event: WindowEvent::CloseRequested,
            ..
        } => *control_flow = glutin::event_loop::ControlFlow::Exit,
        Event::LoopDestroyed => unsafe {
            gl_program.take().unwrap().delete(window.gl());
            window.gl().delete_vertex_array(vertex_array);
        },
        event => {
            match event {
                Event::WindowEvent {
                    event: WindowEvent::MouseInput { state, button, .. },
                    ..
                } => {
                    use glutin::event::{ElementState, MouseButton};
                    match (state, button) {
                        (ElementState::Pressed, MouseButton::Left) => {
                            app_state.left_mouse_button_down = true
                        }

                        (ElementState::Released, MouseButton::Left) => {
                            app_state.left_mouse_button_down = false
                        }
                        (ElementState::Pressed, MouseButton::Right) => {
                            app_state.right_mouse_button_down = true
                        }
                        (ElementState::Released, MouseButton::Right) => {
                            app_state.right_mouse_button_down = false
                        }
                        _ => {}
                    }
                }
                Event::WindowEvent {
                    event: WindowEvent::CursorLeft { .. },
                    ..
                } => {
                    app_state.left_mouse_button_down = false;
                    app_state.right_mouse_button_down = false;
                }
                Event::WindowEvent {
                    event: WindowEvent::CursorMoved { position, .. },
                    ..
                } => {
                    app_state.previous_mouse_position = app_state.current_mouse_position;
                    app_state.current_mouse_position = Some(position);
                }
                Event::WindowEvent {
                    event: WindowEvent::Resized(size),
                    ..
                } => {
                    app_state.resolution = size;
                }
                _ => {}
            }
            window.handle_event(event);
        }
    });
}

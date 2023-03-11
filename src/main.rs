use kalimorfia::{
    mouse::MouseState, primitives::color::Color, render::gl_program::GlProgram, window::Window,
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
    pub mouse: MouseState,
    pub resolution: glutin::dpi::PhysicalSize<u32>,
}

fn build_ui(ui: &mut imgui::Ui, _state: &mut State) {
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
        mouse: MouseState::new(),
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
            app_state.mouse.scroll_delta = delta;
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
            if let Event::WindowEvent { ref event, .. } = event {
                app_state.mouse.handle_window_event(event);

                if let WindowEvent::Resized(size) = event {
                    app_state.resolution = *size;
                }
            }

            window.handle_event(event);
        }
    });
}

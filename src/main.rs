use glow::HasContext;
use kalimorfia::{
    math::{
        affine::transforms,
        geometry::{gridable::Gridable, torus::Torus},
    },
    mouse::MouseState,
    primitives::color::Color,
    render::{drawable::Drawable, gl_program::GlProgram, mesh::LineMesh},
    window::Window,
};
use nalgebra::{Matrix4, Point3, RowVector4, Vector3};
use std::{path::Path, time::Instant};

const WINDOW_TITLE: &str = "Kalimorfia";
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
    ui.window("Kalimorfia")
        .size([500.0, 300.0], imgui::Condition::FirstUseEver)
        .position([0.0, 0.0], imgui::Condition::FirstUseEver)
        .build(|| {
            ui.separator();
            ui.text("Imgui works");
        });
}

fn main() {
    let (mut window, event_loop, gl) = Window::new(WINDOW_TITLE, WINDOW_WIDTH, WINDOW_HEIGHT);
    let mut last_frame = Instant::now();

    let mut app_state = State {
        mouse: MouseState::new(),
        resolution: glutin::dpi::PhysicalSize::new(WINDOW_WIDTH, WINDOW_HEIGHT),
    };

    let gl_program = GlProgram::with_shader_paths(
        gl.clone(),
        vec![
            (
                Path::new("shaders/perspective_vertex.glsl"),
                glow::VERTEX_SHADER,
            ),
            (
                Path::new("shaders/simple_fragment.glsl"),
                glow::FRAGMENT_SHADER,
            ),
        ],
    );

    unsafe {
        gl.clear_color(CLEAR_COLOR.r, CLEAR_COLOR.g, CLEAR_COLOR.b, CLEAR_COLOR.a);
    }

    let torus = Torus::with_radii(4.0, 2.0);
    let (vertices, topology) = torus.grid(20, 10);
    let mut mesh = LineMesh::new(gl.clone(), vertices, topology);
    let projection_transform = transforms::projection(std::f32::consts::FRAC_PI_2, 1.0, 0.1, 100.0);
    let view_transform = transforms::look_at(
        Point3::new(0.0, 0.0, 0.0),
        Point3::new(10.0, 10.0, 10.0),
        Vector3::new(0.0, 1.0, 0.0),
    );

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
            unsafe {
                gl.clear(glow::COLOR_BUFFER_BIT);
            }

            mesh.transform(transforms::translate(Vector3::new(0.00, 0.00, -0.03)));

            gl_program
                .uniform_matrix_4_f32_slice("model_transform", mesh.model_transform().as_slice());
            gl_program.uniform_matrix_4_f32_slice("view_transform", view_transform.as_slice());
            gl_program.uniform_matrix_4_f32_slice(
                "projection_transform",
                projection_transform.as_slice(),
            );
            gl_program.enable();
            mesh.draw();
            window.render(&gl, |ui| build_ui(ui, &mut app_state));
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
        event => {
            if let Event::WindowEvent { ref event, .. } = event {
                app_state.mouse.handle_window_event(event);

                if let WindowEvent::Resized(size) = event {
                    app_state.resolution = *size;
                }
            }

            window.handle_event(event, &gl);
        }
    });
}

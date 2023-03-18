use glow::HasContext;
use kalimorfia::{
    camera::Camera,
    math::{
        affine::transforms,
        geometry::{gridable::Gridable, torus::Torus},
    },
    mouse::MouseState,
    primitives::color::Color,
    render::{drawable::Drawable, gl_program::GlProgram, mesh::LineMesh},
    window::Window,
};
use nalgebra::Vector3;
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
    pub torus: Torus,
    pub tube_points: u32,
    pub round_points: u32,
    pub camera: Camera,
    pub torus_changed: bool,
}

macro_rules! safe_slider {
    ($ui:expr, $label:expr, $min:expr, $max:expr, $value:expr) => {
        $ui.slider_config($label, $min, $max)
            .flags(imgui::SliderFlags::NO_INPUT)
            .build($value)
    };
}

fn build_ui(ui: &mut imgui::Ui, state: &mut State) {
    ui.window("Kalimorfia")
        .size([500.0, 300.0], imgui::Condition::FirstUseEver)
        .position([0.0, 0.0], imgui::Condition::FirstUseEver)
        .build(|| {
            ui.separator();
            state.torus_changed |= safe_slider!(ui, "R", 0.1, 10.0, &mut state.torus.inner_radius);
            state.torus_changed |= safe_slider!(ui, "r", 0.1, 10.0, &mut state.torus.tube_radius);
            state.torus_changed |= safe_slider!(ui, "M", 3, 50, &mut state.round_points);
            state.torus_changed |= safe_slider!(ui, "m", 3, 50, &mut state.tube_points);
        });
}

fn main() {
    let (mut window, event_loop, gl) = Window::new(WINDOW_TITLE, WINDOW_WIDTH, WINDOW_HEIGHT);
    let mut last_frame = Instant::now();

    let mut state = State {
        mouse: MouseState::new(),
        resolution: glutin::dpi::PhysicalSize::new(WINDOW_WIDTH, WINDOW_HEIGHT),
        torus: Torus::with_radii(7.0, 2.0),
        tube_points: 10,
        round_points: 10,
        camera: Camera::new(),
        torus_changed: false,
    };

    let (vertices, topology) = state.torus.grid(state.round_points, state.tube_points);
    let mut mesh = LineMesh::new(gl.clone(), vertices, topology);

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

            state.camera.update_from_mouse(&mut state.mouse, &window);

            if state.torus_changed {
                let (vertices, indices) = state.torus.grid(state.round_points, state.tube_points);
                mesh.update_vertices(vertices, indices);
            }

            let view_transform = state.camera.view_transform();

            let projection_transform = transforms::projection(
                std::f32::consts::FRAC_PI_2,
                state.resolution.width as f32 / state.resolution.height as f32,
                0.1,
                100.0,
            );

            gl_program
                .uniform_matrix_4_f32_slice("model_transform", mesh.model_transform().as_slice());
            gl_program.uniform_matrix_4_f32_slice("view_transform", view_transform.as_slice());
            gl_program.uniform_matrix_4_f32_slice(
                "projection_transform",
                projection_transform.as_slice(),
            );
            gl_program.enable();
            mesh.draw();
            window.render(&gl, |ui| build_ui(ui, &mut state));
        }
        Event::WindowEvent {
            event: WindowEvent::CloseRequested,
            ..
        } => *control_flow = glutin::event_loop::ControlFlow::Exit,
        event => {
            if let Event::WindowEvent { ref event, .. } = event {
                state.mouse.handle_window_event(event);

                if let WindowEvent::Resized(size) = event {
                    state.resolution = *size;
                }
            }

            window.handle_event(event, &gl);
        }
    });
}

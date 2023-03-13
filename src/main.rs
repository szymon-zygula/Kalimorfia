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
    pub torus: Torus,
    pub tube_points: u32,
    pub round_points: u32,
    pub horizontal_view_angle: f32,
    pub vertical_view_angle: f32,
    pub camera_distance: f32,
    pub cursor_position: Vector3<f32>,
}

fn build_ui(ui: &mut imgui::Ui, state: &mut State) {
    ui.window("Kalimorfia")
        .size([500.0, 300.0], imgui::Condition::FirstUseEver)
        .position([0.0, 0.0], imgui::Condition::FirstUseEver)
        .build(|| {
            ui.separator();
            ui.slider("R", 0.1, 10.0, &mut state.torus.inner_radius);
            ui.slider("r", 0.1, 10.0, &mut state.torus.tube_radius);
            ui.slider("M", 3, 50, &mut state.round_points);
            ui.slider("m", 3, 50, &mut state.tube_points);
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
        horizontal_view_angle: 0.0,
        vertical_view_angle: 0.0,
        camera_distance: 10.0,
        cursor_position: Vector3::new(0.0, 0.0, 0.0),
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

            let mouse_delta = state.mouse.position_delta();

            if !window.imgui_using_mouse() {
                if state.mouse.is_middle_button_down() {
                    state.horizontal_view_angle += mouse_delta.x as f32 * 0.05;
                    state.vertical_view_angle += mouse_delta.y as f32 * 0.05;
                }

                state.camera_distance -= state.mouse.scroll_delta();

                if state.camera_distance < 0.0 {
                    state.camera_distance = 0.0;
                }
            }

            let view_transform = (transforms::translate(state.cursor_position)
                * transforms::rotate_y(-state.horizontal_view_angle)
                * transforms::rotate_x(-state.vertical_view_angle)
                * transforms::translate(Vector3::new(0.0, 0.0, state.camera_distance)))
            .try_inverse()
            .unwrap();

            let projection_transform = transforms::projection(
                std::f32::consts::FRAC_PI_2,
                state.resolution.width as f32 / state.resolution.height as f32,
                0.1,
                100.0,
            );

            let (vertices, topology) = state.torus.grid(state.round_points, state.tube_points);
            let mesh = LineMesh::new(gl.clone(), vertices, topology);

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

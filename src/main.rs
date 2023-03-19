use glow::HasContext;
use glutin::platform::run_return::EventLoopExtRunReturn;
use kalimorfia::{
    camera::Camera,
    entities::{
        cursor::Cursor,
        entity::{Entity, SceneObject},
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

struct State<'gl> {
    pub torus: Torus<'gl>,
    pub cursor: Cursor<'gl>,
    pub camera: Camera,
}

fn build_ui(ui: &mut imgui::Ui, state: &mut State) {
    ui.window("Main control")
        .size([500.0, 200.0], imgui::Condition::FirstUseEver)
        .position([0.0, 0.0], imgui::Condition::FirstUseEver)
        .build(|| {
            ui.separator();
            ui.text("Object creation");
            if ui.button("Torus") {}

            if ui.button("Point") {}

            ui.separator();
            ui.text("Cursor control");
            state.cursor.control_ui(ui);

            if ui.button("Center on cursor") {
                state.camera.center = state.cursor.position();
            }
        });

    ui.window("Selected object")
        .size([500.0, 500.0], imgui::Condition::FirstUseEver)
        .position([0.0, 200.0], imgui::Condition::FirstUseEver)
        .build(|| {
            state.torus.control_ui(ui);
        });
}

fn main() {
    let (mut window, mut event_loop, gl) = Window::new(WINDOW_TITLE, WINDOW_WIDTH, WINDOW_HEIGHT);
    let mut last_frame = Instant::now();
    let mut resolution = glutin::dpi::PhysicalSize::new(WINDOW_WIDTH, WINDOW_HEIGHT);
    let mut mouse = MouseState::new();
    let grid = SceneGrid::new(&gl, 80, 40.0);
    let mut state = State {
        camera: Camera::new(),
        torus: Torus::new(&gl),
        cursor: Cursor::new(&gl, 1.0),
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
            state.torus.draw(&projection_transform, &view_transform);
            state.cursor.draw(&projection_transform, &view_transform);
            window.render(&gl, |ui| build_ui(ui, &mut state));
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

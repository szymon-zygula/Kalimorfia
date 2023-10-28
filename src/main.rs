mod json;
mod main_control;
mod shaders;
mod state;
mod path_gen_ui;

use crate::{main_control::MainControl, state::State};
use glow::HasContext;
use glutin::platform::run_return::EventLoopExtRunReturn;
use kalimorfia::{
    camera::Camera,
    constants::*,
    entities::{
        entity::{DrawType, Drawable},
        manager::EntityManager,
        scene_grid::SceneGrid,
    },
    mouse::MouseState,
    render::stereo,
    window::Window,
};
use nalgebra::Matrix4;
use std::{cell::RefCell, rc::Rc, time::Instant};

#[derive(PartialEq)]
enum SelectResult {
    Select,
    Unselect,
    Nothing,
}

fn select_clicked(
    pixel: glutin::dpi::PhysicalPosition<u32>,
    state: &mut State,
    entity_manager: &RefCell<EntityManager>,
) -> SelectResult {
    let point = state.camera.screen_to_ndc(&pixel);
    let mut closest_id = None;
    let mut closest_dist = f32::INFINITY;

    for (&id, entity) in entity_manager.borrow().entities() {
        if let Some(camera_distance) =
            entity
                .borrow_mut()
                .is_at_ndc(point, &state.camera, entity_manager.borrow().entities())
        {
            if camera_distance < closest_dist {
                closest_dist = camera_distance;
                closest_id = Some(id);
            }
        }
    }

    if let Some(id) = closest_id {
        if state.selector.toggle(id) {
            SelectResult::Select
        } else {
            SelectResult::Unselect
        }
    } else {
        SelectResult::Nothing
    }
}

fn render_scene(
    gl: &glow::Context,
    state: &State,
    camera: &Camera,
    entity_manager: &RefCell<EntityManager>,
    grid: &SceneGrid,
) {
    unsafe {
        gl.clear(glow::COLOR_BUFFER_BIT | glow::DEPTH_BUFFER_BIT);
    }

    if !state.gk_mode {
        grid.draw_regular(camera);
    }

    for (&id, &selected) in state.selector.selectables() {
        if state.gk_mode && entity_manager.borrow().get_entity(id).is_single_point() {
            continue;
        }

        entity_manager.borrow().draw_referential(
            id,
            camera,
            &Matrix4::identity(),
            if selected {
                DrawType::Selected
            } else {
                DrawType::Regular
            },
        );
    }

    entity_manager.borrow().draw_referential(
        state.selected_aggregate_id,
        camera,
        &Matrix4::identity(),
        DrawType::Regular,
    );

    state.cursor.draw_regular(camera);
}

fn update_io(
    state: &mut State,
    window: &Window,
    mouse: &mut MouseState,
    prevent_grab: &mut bool,
    entity_manager: &RefCell<EntityManager>,
) {
    if state.camera.update_from_mouse(mouse, window) {
        state.cursor.set_camera(&state.camera);
    }

    if !window.imgui_using_mouse() && mouse.has_left_button_been_pressed() {
        if let Some(position) = mouse.integer_position() {
            if select_clicked(position, state, entity_manager) == SelectResult::Unselect {
                *prevent_grab = true;
            }
        }
    }

    if !mouse.is_left_button_down() {
        *prevent_grab = false;
    }

    if !window.imgui_using_mouse() && mouse.is_left_button_down() && !*prevent_grab {
        if let Some(only_selected) = state.selector.only_selected() {
            if let Some(position) = &mouse.integer_position() {
                entity_manager.borrow_mut().set_ndc(
                    only_selected,
                    &state.camera.screen_to_ndc(position),
                    &state.camera,
                );
            }
        }
    }
}

fn main() {
    let (mut window, mut event_loop, gl) = Window::new(WINDOW_TITLE, WINDOW_WIDTH, WINDOW_HEIGHT);
    let mut last_frame = Instant::now();
    let mut mouse = MouseState::new();
    let grid = SceneGrid::new(&gl, 100, 50.0);
    let shader_manager = shaders::create_shader_manager(&gl);
    let entity_manager = RefCell::new(EntityManager::new());
    let mut state = State::new(&gl, &entity_manager, Rc::clone(&shader_manager));
    let mut main_control = MainControl::new(Rc::clone(&shader_manager), &entity_manager, &gl);
    let mut prevent_grab = false;

    unsafe {
        gl.clear_color(CLEAR_COLOR.r, CLEAR_COLOR.g, CLEAR_COLOR.b, CLEAR_COLOR.a);
        gl.clear_depth_f32(-10000000000000.0);
        gl.enable(glow::DEPTH_TEST);
        gl.depth_func(glow::GREATER);
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
            update_io(
                &mut state,
                &window,
                &mut mouse,
                &mut prevent_grab,
                &entity_manager,
            );

            if let Some((left_camera, right_camera)) = state.camera.stereo_cameras() {
                stereo::draw(&gl, &left_camera, &right_camera, |camera| {
                    render_scene(&gl, &state, camera, &entity_manager, &grid)
                });
            } else {
                render_scene(&gl, &state, &state.camera, &entity_manager, &grid);
            }

            window.render(&gl, |ui| main_control.build_ui(ui, &mut state));
        }
        Event::WindowEvent {
            event: WindowEvent::CloseRequested,
            ..
        } => *control_flow = glutin::event_loop::ControlFlow::Exit,
        event => {
            if let Event::WindowEvent { event, .. } = &event {
                mouse.handle_window_event(event);

                if let WindowEvent::Resized(resolution) = event {
                    state.camera.resolution = *resolution;
                    state
                        .cursor
                        .set_camera_and_resolution(&state.camera, resolution);
                }
            }

            window.handle_event(event, &gl);
        }
    });
}

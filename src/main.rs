mod constants;
mod main_control;
mod shaders;
mod state;

use crate::{constants::*, main_control::MainControl, state::State};
use glow::HasContext;
use glutin::platform::run_return::EventLoopExtRunReturn;
use kalimorfia::{
    entities::{
        entity::{DrawType, Drawable},
        manager::EntityManager,
        scene_grid::SceneGrid,
    },
    mouse::MouseState,
    window::Window,
};
use nalgebra::Matrix4;
use std::{cell::RefCell, rc::Rc, time::Instant};

fn select_clicked(
    pixel: glutin::dpi::PhysicalPosition<u32>,
    state: &mut State,
    resolution: &glutin::dpi::PhysicalSize<u32>,
    entity_manager: &RefCell<EntityManager>,
) -> bool {
    let point = state.camera.screen_to_ndc(&pixel);
    let mut closest_id = None;
    let mut closest_dist = f32::INFINITY;

    for (&id, entity) in entity_manager.borrow().entities() {
        let (is_at_point, camera_distance) = entity.borrow().is_at_point(
            point,
            &state.camera.projection_transform(),
            &state.camera.view_transform(),
            resolution,
        );

        if is_at_point && camera_distance < closest_dist {
            closest_dist = camera_distance;
            closest_id = Some(id);
        }
    }

    if let Some(id) = closest_id {
        state.selector.toggle(id)
    } else {
        false
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
    let main_control = MainControl::new(Rc::clone(&shader_manager), &entity_manager, &gl);
    let mut grabbing = false;

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

            if state.camera.update_from_mouse(&mut mouse, &window) {
                state.cursor.set_camera(&state.camera);
            }

            if !window.imgui_using_mouse() && mouse.has_left_button_been_pressed() {
                if let Some(position) = mouse.integer_position() {
                    if select_clicked(position, &mut state, &window.size(), &entity_manager) {
                        grabbing = true;
                    }
                }
            }

            if !mouse.is_left_button_down() {
                grabbing = false;
            }

            if grabbing {
                if let Some(only_selected) = state.selector.only_selected() {
                    if let Some(ref position) = mouse.integer_position() {
                        entity_manager.borrow_mut().set_ndc(
                            only_selected,
                            &state.camera.screen_to_ndc(position),
                            &state.camera,
                        );
                    }
                }
            }

            grid.draw_regular(&state.camera);

            for (&id, &selected) in state.selector.selectables() {
                entity_manager.borrow().draw_referential(
                    id,
                    &state.camera,
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
                &state.camera,
                &Matrix4::identity(),
                DrawType::Regular,
            );

            state.cursor.draw_regular(&state.camera);
            window.render(&gl, |ui| main_control.build_ui(ui, &mut state));
        }
        Event::WindowEvent {
            event: WindowEvent::CloseRequested,
            ..
        } => *control_flow = glutin::event_loop::ControlFlow::Exit,
        event => {
            if let Event::WindowEvent { ref event, .. } = event {
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

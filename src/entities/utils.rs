use super::entity::ReferentialSceneEntity;
use crate::camera::Camera;
use nalgebra::{Point3, Vector2};
use std::{
    cell::RefCell,
    collections::{BTreeMap, HashMap, HashSet},
};

pub fn segregate_points<'gl>(
    entities: &BTreeMap<usize, RefCell<Box<dyn ReferentialSceneEntity<'gl> + 'gl>>>,
    selected: &[usize],
) -> Vec<(usize, String, bool)> {
    let selected_name_selection = selected
        .iter()
        .map(|id| (*id, entities[id].borrow().name(), true));

    let not_selected_name_selection = entities
        .iter()
        .filter(|(id, entity)| {
            !selected.contains(id)
                && entity
                    .try_borrow()
                    .map_or(false, |entity| entity.is_single_point())
        })
        .map(|(id, entity)| (*id, entity.borrow().name(), false));

    selected_name_selection
        .chain(not_selected_name_selection)
        .collect()
}

pub fn update_point_subs(
    selection: Vec<(usize, bool)>,
    controller_id: usize,
    subscriptions: &mut HashMap<usize, HashSet<usize>>,
) {
    for (id, selected) in selection {
        if selected {
            subscriptions.get_mut(&controller_id).unwrap().insert(id);
        } else {
            subscriptions.get_mut(&controller_id).unwrap().remove(&id);
        }
    }
}

pub fn polygon_pixel_length_direct<'gl>(points: &[Point3<f32>], camera: &Camera) -> f32 {
    let mut sum = 0.0;
    for i in 1..points.len() {
        let point1 = camera.projection_transform()
            * camera.view_transform()
            * points[i - 1].to_homogeneous();
        let point2 =
            camera.projection_transform() * camera.view_transform() * points[i].to_homogeneous();

        let diff = Point3::from_homogeneous(point1)
            .unwrap_or(Point3::origin())
            .xy()
            - Point3::from_homogeneous(point2)
                .unwrap_or(Point3::origin())
                .xy();

        let clamped_point = Vector2::new(diff.x.clamp(-1.0, 1.0), diff.y.clamp(-1.0, 1.0));
        sum += clamped_point
            .component_mul(&Vector2::new(
                0.5 * camera.resolution.width as f32,
                0.5 * camera.resolution.height as f32,
            ))
            .norm();
    }

    sum
}

pub fn polygon_pixel_length<'gl>(
    points: &[usize],
    entities: &BTreeMap<usize, RefCell<Box<dyn ReferentialSceneEntity<'gl> + 'gl>>>,
    camera: &Camera,
) -> f32 {
    polygon_pixel_length_direct(
        &points
            .iter()
            .map(|id| entities[id].borrow().location().unwrap())
            .collect::<Vec<Point3<f32>>>(),
        camera,
    )
}

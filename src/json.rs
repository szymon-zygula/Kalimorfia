use crate::state::State;
use kalimorfia::{
    entities::{
        bezier_surface_args::BezierSurfaceArgs, bezier_surface_c0::BezierSurfaceC0,
        bezier_surface_c2::BezierSurfaceC2, cubic_spline_c0::CubicSplineC0,
        cubic_spline_c2::CubicSplineC2, manager::EntityManager, point::Point, torus::Torus,
    },
    render::shader_manager::ShaderManager,
};
use nalgebra::Point3;
use serde::{Deserialize, Serialize};
use std::rc::Rc;

fn add_ids_to_surface(free_id: &mut usize, obj: &mut serde_json::Map<String, serde_json::Value>) {
    // Strange, but is there an alternative???
    let maybe_surface = if let Some(inside) = obj.get_mut("bezierSurfaceC0") {
        Some(inside)
    } else {
        obj.get_mut("bezierSurfaceC2")
    };

    if let Some(serde_json::Value::Object(surface)) = maybe_surface {
        let Some(serde_json::Value::Array(patches)) = surface.get_mut("patches") else {
                    panic!("No patches in surface JSON");
                };

        for patch in patches {
            let serde_json::Value::Object(patch) = patch else {
                        panic!("Error in surface JSON");
                    };

            patch.insert(String::from("id"), serde_json::json!(free_id));
            *free_id += 1;
        }
    }
}

pub fn serialize_scene(entity_manager: &EntityManager, state: &State) -> serde_json::Value {
    let mut points = Vec::new();
    let mut others = Vec::new();
    let mut free_id = entity_manager.next_id();

    for &id in state.selector.selectables().keys() {
        let manager = entity_manager;
        let entity = manager.get_entity(id);
        let mut json = entity.to_json();

        if let serde_json::Value::Object(obj) = &mut json {
            obj.insert(String::from("id"), serde_json::json!(id));
            add_ids_to_surface(&mut free_id, obj);
        } else {
            panic!("Something has been deserialized to something else than an object");
        }

        if entity.is_single_point() {
            points.push(json);
        } else {
            others.push(json);
        }
    }

    serde_json::json!({
        "points": points,
        "geometry": others
    })
}

#[derive(Serialize, Deserialize)]
struct Position {
    x: f32,
    y: f32,
    z: f32,
}

#[derive(Serialize, Deserialize)]
struct JTorus {

}

#[derive(Serialize, Deserialize)]
struct JBezierC0 {

}

#[derive(Serialize, Deserialize)]
struct JBezierC2 {

}

#[derive(Serialize, Deserialize)]
struct JInterpolatedC2 {

}

#[derive(Serialize, Deserialize)]
struct JPatch {

}

#[derive(Serialize, Deserialize)]
struct JBezierSurfaceC0 {

}

#[derive(Serialize, Deserialize)]
struct JBezierSurfaceC2 {

}

pub fn deserialize_position(json: serde_json::Value) -> Result<Position, ()> {
    let serde_json::Value::Object(position) = json else { return Err(()); };
    let Some(serde_json::Value::Number(x)) = position.get("x") else { return Err(()); };
    let Some(serde_json::Value::Number(y)) = position.get("y") else { return Err(()); };
    let Some(serde_json::Value::Number(z)) = position.get("z") else { return Err(()); };

    Ok(Position {
        x: x.as_f64().ok_or(())? as f32,
        y: y.as_f64().ok_or(())? as f32,
        z: z.as_f64().ok_or(())? as f32,
    })
}

pub fn deserialize_scene<'gl>(
    gl: &'gl glow::Context,
    shader_manager: &Rc<ShaderManager<'gl>>,
    json: serde_json::Value,
    entity_manager: &mut EntityManager<'gl>,
    state: &mut State,
) -> Result<(), ()> {
    let serde_json::Value::Object(obj) = json else { return Err(()); };
    let Some(serde_json::Value::Array(geometry)) = obj.get("geometry") else { return Err(()); };
    let Some(serde_json::Value::Array(points)) = obj.get("points") else { return Err(()); };

    for point in points {
        let serde_json::Value::Object(point) = point else { return Err(()); };
        let Some(serde_json::Value::Number(id)) = point.get("id") else { return Err(()); };
        let id = id.as_u64().ok_or(())? as usize;
        let Some(position) = point.get("position") else { return Err(()); };
        let position: Position = serde_json::from_value(position.clone()).map_err(|_| ())?;

        let point = Box::new(Point::with_position(
            gl,
            Point3::new(position.x, position.y, position.z),
            Rc::clone(&state.name_repo),
            Rc::clone(&shader_manager),
        ));

        entity_manager.add_entity_with_id(point, id);
        state.selector.add_selectable(id);
    }

    for object in geometry {
        let serde_json::Value::Object(object) = object else { return Err(()); };
        let Some(serde_json::Value::Number(id)) = object.get("id") else { return Err(()); };
        let id = id.as_u64().ok_or(())? as usize;
        let Some(serde_json::Value::String(type_)) = object.get("type") else { return Err(()); };

        let entity = match type_.as_str() {
            "torus" => {
                let torus = Box::new(Torus::new(
                    gl,
                    Rc::clone(&state.name_repo),
                    Rc::clone(shader_manager),
                ));

                torus
            }
            "bezierC0" => {}
            "bezierC2" => {}
            "interpolatedC2" => {}
            "bezierSurfaceC0" => {}
            "bezierSurfaceC2" => {}
            _ => return Err(()),
        };

        entity_manager.add_entity_with_id(entity, id);
        state.selector.add_selectable(id);
    }

    Ok(())
}

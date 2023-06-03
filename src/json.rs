use crate::state::State;
use kalimorfia::{
    entities::{
        basic::{LinearTransformEntity, Orientation, Scale, Shear, Translation},
        bezier_surface_args::{BezierCylinderArgs, BezierFlatSurfaceArgs, BezierSurfaceArgs},
        bezier_surface_c0::BezierSurfaceC0,
        bezier_surface_c2::BezierSurfaceC2,
        cubic_spline_c0::CubicSplineC0,
        cubic_spline_c2::CubicSplineC2,
        entity::ReferentialSceneEntity,
        interpolating_spline::InterpolatingSpline,
        manager::EntityManager,
        point::Point,
        torus::Torus,
    },
    math::{affine::transforms, decompositions::axis_angle::AxisAngleDecomposition},
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
struct XYZ {
    x: f32,
    y: f32,
    z: f32,
}

impl XYZ {
    pub fn point(&self) -> Point3<f32> {
        Point3::new(self.x, self.y, self.z)
    }

    pub fn rotation(&self) -> Orientation {
        let rotation = transforms::rotate_z(self.z)
            * transforms::rotate_y(self.y)
            * transforms::rotate_x(self.x);
        let decomp = AxisAngleDecomposition::decompose(&rotation);

        Orientation {
            angle: decomp.angle,
            axis: decomp.axis,
        }
    }

    pub fn shear(&self) -> Shear {
        Shear {
            xy: self.x,
            xz: self.y,
            yz: self.z,
        }
    }

    pub fn scale(&self) -> Scale {
        Scale {
            scale: self.point().coords,
        }
    }

    pub fn translation(&self) -> Translation {
        Translation {
            translation: self.point().coords,
        }
    }
}

#[derive(Serialize, Deserialize)]
struct XY {
    x: usize,
    y: usize,
}

#[derive(Serialize, Deserialize)]
struct PointRef {
    id: usize,
}

#[derive(Serialize, Deserialize)]
struct JTorus {
    #[serde(rename = "objectType")]
    object_type: String,
    name: String,
    id: usize,
    position: XYZ,
    rotation: XYZ,
    scale: XYZ,
    shear: Option<XYZ>,
    samples: XY,
    #[serde(rename = "smallRadius")]
    small_radius: f32,
    #[serde(rename = "largeRadius")]
    large_radius: f32,
}

#[derive(Serialize, Deserialize)]
struct JBezierC0 {
    #[serde(rename = "objectType")]
    object_type: String,
    name: String,
    id: usize,
    #[serde(rename = "controlPoints")]
    control_points: Vec<PointRef>,
}

#[derive(Serialize, Deserialize)]
struct JBezierC2 {
    #[serde(rename = "objectType")]
    object_type: String,
    name: String,
    id: usize,
    #[serde(rename = "deBoorPoints")]
    de_boor_points: Vec<PointRef>,
}

#[derive(Serialize, Deserialize)]
struct JInterpolatedC2 {
    #[serde(rename = "objectType")]
    object_type: String,
    name: String,
    id: usize,
    #[serde(rename = "controlPoints")]
    control_points: Vec<PointRef>,
}

#[derive(Serialize, Deserialize)]
struct JPatch {
    #[serde(rename = "objectType")]
    object_type: String,
    name: String,
    id: usize,
    #[serde(rename = "controlPoints")]
    control_points: [PointRef; 16],
    samples: XY,
}

#[derive(Serialize, Deserialize)]
struct ParameterWrapped {
    u: bool,
    v: bool,
}

#[derive(Serialize, Deserialize)]
struct JBezierSurfaceC0 {
    #[serde(rename = "objectType")]
    object_type: String,
    name: String,
    id: usize,
    patches: Vec<JPatch>,
    #[serde(rename = "parameterWrapped")]
    parameter_wrapped: ParameterWrapped,
    size: XY,
}

impl JBezierSurfaceC0 {
    pub fn control_points(&self) -> Vec<Vec<usize>> {
        let mut points = Vec::new();

        for u in 0..self.size.x {
            points.push(Vec::new());

            for v in 0..(self.size.y - 1) {
                let patch_u = u / 3;
                let patch_v = v / 3;
                let patch_idx = patch_v * self.size.y + patch_u;
                let local_u = u % 3;
                let local_v = v % 3;
                let point_idx = local_v * 4 + local_u;
                points
                    .last_mut()
                    .unwrap()
                    .push(self.patches[patch_idx].control_points[point_idx].id);
            }
        }

        points
    }

    pub fn args(&self) -> BezierSurfaceArgs {
        if self.parameter_wrapped.u {
            BezierSurfaceArgs::Cylinder(BezierCylinderArgs {
                length: 0.0,
                radius: 0.0,
                around_patches: self.size.x as i32,
                along_patches: self.size.y as i32,
            })
        } else {
            BezierSurfaceArgs::Surface(BezierFlatSurfaceArgs {
                x_length: 0.0,
                z_length: 0.0,
                x_patches: self.size.x as i32,
                z_patches: self.size.y as i32,
            })
        }
    }
}

#[derive(Serialize, Deserialize)]
struct JBezierSurfaceC2 {
    #[serde(rename = "objectType")]
    object_type: String,
    name: String,
    id: usize,
    patches: Vec<JPatch>,
    #[serde(rename = "parameterWrapped")]
    parameter_wrapped: ParameterWrapped,
    size: XY,
}

impl JBezierSurfaceC2 {
    pub fn control_points(&self) -> Vec<Vec<usize>> {
        let points = Vec::new();

        points
    }

    pub fn args(&self) -> BezierSurfaceArgs {
        if self.parameter_wrapped.u {
            BezierSurfaceArgs::Cylinder(BezierCylinderArgs {
                length: 0.0,
                radius: 0.0,
                around_patches: self.size.x as i32,
                along_patches: self.size.y as i32,
            })
        } else {
            BezierSurfaceArgs::Surface(BezierFlatSurfaceArgs {
                x_length: 0.0,
                z_length: 0.0,
                x_patches: self.size.x as i32,
                z_patches: self.size.y as i32,
            })
        }
    }
}

pub fn deserialize_position(json: serde_json::Value) -> Result<XYZ, ()> {
    let serde_json::Value::Object(position) = json else { return Err(()); };
    let Some(serde_json::Value::Number(x)) = position.get("x") else { return Err(()); };
    let Some(serde_json::Value::Number(y)) = position.get("y") else { return Err(()); };
    let Some(serde_json::Value::Number(z)) = position.get("z") else { return Err(()); };

    Ok(XYZ {
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
    state: &mut State<'gl, '_>,
) -> Result<(), ()> {
    let serde_json::Value::Object(obj) = json else { return Err(()); };
    let Some(serde_json::Value::Array(geometry)) = obj.get("geometry") else { return Err(()); };
    let Some(serde_json::Value::Array(points)) = obj.get("points") else { return Err(()); };

    for point in points {
        let serde_json::Value::Object(point) = point else { return Err(()); };
        let Some(serde_json::Value::Number(id)) = point.get("id") else { return Err(()); };
        let id = id.as_u64().ok_or(())? as usize;
        let Some(position) = point.get("position") else { return Err(()); };
        let position: XYZ = serde_json::from_value(position.clone()).map_err(|_| ())?;

        let point = Box::new(Point::with_position(
            gl,
            Point3::new(position.x, position.y, position.z),
            Rc::clone(&state.name_repo),
            Rc::clone(&shader_manager),
        ));

        entity_manager.add_entity_with_id(point, id);
        state.selector.add_selectable(id);
    }

    for geom in geometry {
        let serde_json::Value::Object(object) = geom else { return Err(()); };
        let Some(serde_json::Value::String(type_)) = object.get("type") else { return Err(()); };
        let Some(serde_json::Value::Number(id)) = object.get("id") else { return Err(()); };
        let id = id.as_u64().ok_or(())? as usize;

        let entity: Box<dyn ReferentialSceneEntity<'gl>> = match type_.as_str() {
            "torus" => torus_from_json(gl, state, shader_manager, geom.clone())?,
            "bezierC0" => {
                bezier_c0_from_json(gl, id, state, shader_manager, entity_manager, geom.clone())?
            }
            "bezierC2" => {
                bezier_c2_from_json(gl, id, state, shader_manager, entity_manager, geom.clone())?
            }
            "interpolatedC2" => interpolating_from_json(
                gl,
                id,
                state,
                shader_manager,
                entity_manager,
                geom.clone(),
            )?,
            "bezierSurfaceC0" => {
                surface_c0_from_json(gl, id, state, shader_manager, entity_manager, geom.clone())?
            }

            "bezierSurfaceC2" => {
                surface_c2_from_json(gl, id, state, shader_manager, entity_manager, geom.clone())?
            }
            _ => return Err(()),
        };

        entity_manager.add_entity_with_id(entity, id);
        state.selector.add_selectable(id);
    }

    Ok(())
}

fn torus_from_json<'gl>(
    gl: &'gl glow::Context,
    state: &State<'gl, '_>,
    shader_manager: &Rc<ShaderManager<'gl>>,
    geom: serde_json::Value,
) -> Result<Box<Torus<'gl>>, ()> {
    let jtorus: JTorus = serde_json::from_value(geom).map_err(|_| ())?;
    let mut torus = Box::new(Torus::new(
        gl,
        Rc::clone(&state.name_repo),
        Rc::clone(shader_manager),
    ));

    let mut tref = &mut torus.as_mut();
    let mut trans = &mut tref.linear_transform;
    trans.translation = jtorus.position.translation();
    trans.orientation = jtorus.rotation.rotation();
    trans.scale = jtorus.scale.scale();
    trans.shear = jtorus.shear.map_or(
        Shear {
            xy: 0.0,
            xz: 0.0,
            yz: 0.0,
        },
        |s| s.shear(),
    );

    tref.torus.inner_radius = jtorus.large_radius as f64;
    tref.torus.tube_radius = jtorus.small_radius as f64;
    tref.tube_points = jtorus.samples.x as u32;
    tref.round_points = jtorus.samples.y as u32;

    Ok(torus)
}

fn bezier_c0_from_json<'gl>(
    gl: &'gl glow::Context,
    id: usize,
    state: &State<'gl, '_>,
    shader_manager: &Rc<ShaderManager<'gl>>,
    entity_manager: &mut EntityManager<'gl>,
    geom: serde_json::Value,
) -> Result<Box<CubicSplineC0<'gl>>, ()> {
    let spline: JBezierC0 = serde_json::from_value(geom.clone()).map_err(|_| ())?;
    let points: Vec<_> = spline.control_points.iter().map(|p| p.id).collect();
    let spline = Box::new(CubicSplineC0::through_points(
        gl,
        Rc::clone(&state.name_repo),
        Rc::clone(&shader_manager),
        points.clone(),
        entity_manager.entities(),
    ));

    for point in points {
        entity_manager.subscribe(id, point);
    }

    Ok(spline)
}

fn bezier_c2_from_json<'gl>(
    gl: &'gl glow::Context,
    id: usize,
    state: &State<'gl, '_>,
    shader_manager: &Rc<ShaderManager<'gl>>,
    entity_manager: &mut EntityManager<'gl>,
    geom: serde_json::Value,
) -> Result<Box<CubicSplineC2<'gl>>, ()> {
    let spline: JBezierC2 = serde_json::from_value(geom.clone()).map_err(|_| ())?;
    let points: Vec<_> = spline.de_boor_points.iter().map(|p| p.id).collect();
    let spline = Box::new(CubicSplineC2::through_points(
        gl,
        Rc::clone(&state.name_repo),
        Rc::clone(&shader_manager),
        points.clone(),
        entity_manager.entities(),
    ));

    for point in points {
        entity_manager.subscribe(id, point);
    }

    Ok(spline)
}

fn interpolating_from_json<'gl>(
    gl: &'gl glow::Context,
    id: usize,
    state: &State<'gl, '_>,
    shader_manager: &Rc<ShaderManager<'gl>>,
    entity_manager: &mut EntityManager<'gl>,
    geom: serde_json::Value,
) -> Result<Box<InterpolatingSpline<'gl>>, ()> {
    let spline: JInterpolatedC2 = serde_json::from_value(geom.clone()).map_err(|_| ())?;
    let points: Vec<_> = spline.control_points.iter().map(|p| p.id).collect();
    let spline = Box::new(InterpolatingSpline::through_points(
        gl,
        Rc::clone(&state.name_repo),
        Rc::clone(&shader_manager),
        points.clone(),
        entity_manager.entities(),
    ));

    for point in points {
        entity_manager.subscribe(id, point);
    }

    Ok(spline)
}

fn surface_c0_from_json<'gl>(
    gl: &'gl glow::Context,
    id: usize,
    state: &State<'gl, '_>,
    shader_manager: &Rc<ShaderManager<'gl>>,
    entity_manager: &mut EntityManager<'gl>,
    geom: serde_json::Value,
) -> Result<Box<BezierSurfaceC0<'gl>>, ()> {
    let surface: JBezierSurfaceC0 = serde_json::from_value(geom.clone()).map_err(|_| ())?;
    let points = surface.control_points();

    let surface = Box::new(BezierSurfaceC0::new(
        gl,
        Rc::clone(&state.name_repo),
        Rc::clone(&shader_manager),
        points.clone(),
        entity_manager.entities(),
        surface.args(),
    ));

    for &point in points.iter().flatten() {
        entity_manager.subscribe(id, point);
    }

    Ok(surface)
}

fn surface_c2_from_json<'gl>(
    gl: &'gl glow::Context,
    id: usize,
    state: &State<'gl, '_>,
    shader_manager: &Rc<ShaderManager<'gl>>,
    entity_manager: &mut EntityManager<'gl>,
    geom: serde_json::Value,
) -> Result<Box<BezierSurfaceC2<'gl>>, ()> {
    let surface: JBezierSurfaceC2 = serde_json::from_value(geom.clone()).map_err(|_| ())?;
    let points = surface.control_points();

    let surface = Box::new(BezierSurfaceC2::new(
        gl,
        Rc::clone(&state.name_repo),
        Rc::clone(&shader_manager),
        points.clone(),
        entity_manager.entities(),
        surface.args(),
    ));

    for &point in points.iter().flatten() {
        entity_manager.subscribe(id, point);
    }

    Ok(surface)
}

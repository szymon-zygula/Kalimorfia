use crate::state::State;
use kalimorfia::{
    camera::Camera,
    entities::{
        basic::{Orientation, Scale, Shear, Translation},
        bezier_surface_args::{BezierCylinderArgs, BezierFlatSurfaceArgs, BezierSurfaceArgs},
        bezier_surface_c0::BezierSurfaceC0,
        bezier_surface_c2::BezierSurfaceC2,
        cubic_spline_c0::CubicSplineC0,
        cubic_spline_c2::CubicSplineC2,
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
    let Some(serde_json::Value::String(object_type)) = obj.get("objectType") else { return; };

    if object_type != "bezierSurfaceC0" && object_type != "bezierSurfaceC2" {
        return;
    }

    let Some(serde_json::Value::Array(patches)) = obj.get_mut("patches") else {
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

pub fn serialize_scene(entity_manager: &EntityManager, state: &State) -> serde_json::Value {
    let camera = state.camera.to_json();
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
        "camera": camera,
        "points": points,
        "geometry": others
    })
}

#[derive(Serialize, Deserialize)]
struct Xyz {
    x: f32,
    y: f32,
    z: f32,
}

impl Xyz {
    pub fn point(&self) -> Point3<f32> {
        Point3::new(self.x, self.y, self.z)
    }

    pub fn rotation(&self) -> Orientation {
        let rotation = transforms::rotate_z(self.z)
            * transforms::rotate_y(self.y.to_radians())
            * transforms::rotate_x(self.x.to_radians());
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

#[derive(Serialize, Deserialize, Clone, Copy)]
struct Xy {
    x: usize,
    y: usize,
}

#[derive(Serialize, Deserialize, Clone, Copy)]
struct Xyf {
    x: f32,
    y: f32,
}

#[derive(Serialize, Deserialize, Copy, Clone)]
struct PointRef {
    id: usize,
}

#[derive(Serialize, Deserialize)]
struct JTorus {
    #[serde(rename = "objectType")]
    object_type: String,
    name: Option<String>,
    id: usize,
    position: Xyz,
    rotation: Xyz,
    scale: Xyz,
    shear: Option<Xyz>,
    samples: Xy,
    #[serde(rename = "smallRadius")]
    small_radius: f32,
    #[serde(rename = "largeRadius")]
    large_radius: f32,
}

#[derive(Serialize, Deserialize)]
struct JBezierC0 {
    #[serde(rename = "objectType")]
    object_type: String,
    name: Option<String>,
    id: usize,
    #[serde(rename = "controlPoints")]
    control_points: Vec<PointRef>,
}

#[derive(Serialize, Deserialize)]
struct JBezierC2 {
    #[serde(rename = "objectType")]
    object_type: String,
    name: Option<String>,
    id: usize,
    #[serde(rename = "deBoorPoints")]
    de_boor_points: Vec<PointRef>,
}

#[derive(Serialize, Deserialize)]
struct JInterpolatedC2 {
    #[serde(rename = "objectType")]
    object_type: String,
    name: Option<String>,
    id: usize,
    #[serde(rename = "controlPoints")]
    control_points: Vec<PointRef>,
}

#[derive(Serialize, Deserialize)]
struct JPatch {
    #[serde(rename = "objectType")]
    object_type: String,
    name: Option<String>,
    id: usize,
    #[serde(rename = "controlPoints")]
    control_points: [PointRef; 16],
    samples: Xy,
}

#[derive(Serialize, Deserialize)]
struct ParameterWrapped {
    u: bool,
    v: bool,
}

#[derive(Serialize, Deserialize)]
struct JBezierSurfaceC0 {
    #[serde(rename = "objectType")]
    pub object_type: String,
    pub name: Option<String>,
    pub id: usize,
    pub patches: Vec<JPatch>,
    #[serde(rename = "parameterWrapped")]
    pub parameter_wrapped: ParameterWrapped,
    pub size: Xy,
}

impl JBezierSurfaceC0 {
    pub fn control_points(&self) -> Vec<Vec<usize>> {
        let u_points = self.size.x * 3 + 1;
        let v_points = self.size.y * 3 + 1;
        let mut points: Vec<_> = (0..u_points).map(|_| vec![0; v_points]).collect();

        for u_patch in 0..self.size.x {
            for v_patch in 0..self.size.y {
                for u in 0..4 {
                    for v in 0..4 {
                        let patch_idx = v_patch * self.size.x + u_patch;
                        let point_idx = v * 4 + u;
                        let point_u = u_patch * 3 + u;
                        let point_v = v_patch * 3 + v;

                        points[point_u][point_v] =
                            self.patches[patch_idx].control_points[point_idx].id;
                    }
                }
            }
        }

        if self.parameter_wrapped.u {
            points.pop();
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

    pub fn sampling(&self) -> Xy {
        self.patches[0].samples
    }
}

#[derive(Serialize, Deserialize)]
struct JBezierSurfaceC2 {
    #[serde(rename = "objectType")]
    object_type: String,
    name: Option<String>,
    id: usize,
    patches: Vec<JPatch>,
    #[serde(rename = "parameterWrapped")]
    parameter_wrapped: ParameterWrapped,
    size: Xy,
}

impl JBezierSurfaceC2 {
    pub fn control_points(&self) -> Vec<Vec<usize>> {
        let u_points = self.size.x + 3;
        let v_points = self.size.y + 3;
        let mut points: Vec<_> = (0..u_points).map(|_| vec![0; v_points]).collect();

        for u_patch in 0..self.size.x {
            for v_patch in 0..self.size.y {
                for u in 0..4 {
                    for v in 0..4 {
                        let patch_idx = v_patch * self.size.x + u_patch;
                        let point_idx = v * 4 + u;
                        let point_u = u_patch + u;
                        let point_v = v_patch + v;

                        points[point_u][point_v] =
                            self.patches[patch_idx].control_points[point_idx].id;
                    }
                }
            }
        }

        if self.parameter_wrapped.u {
            points.pop();
            points.pop();
            points.pop();
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

    pub fn sampling(&self) -> Xy {
        self.patches[0].samples
    }
}

#[derive(Serialize, Deserialize)]
struct JCamera {
    #[serde(rename = "focusPoint")]
    pub focus_point: Xyz,
    pub distance: f32,
    pub rotation: Xyf,
}

impl JCamera {
    fn new() -> Self {
        Self {
            focus_point: Xyz {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
            distance: 1.0,
            rotation: Xyf { x: 0.0, y: 0.0 },
        }
    }
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
    let mut max_id = entity_manager.next_id() as isize - 1;

    let camera = obj.get("camera");
    camera_json(&mut state.camera, camera)?;

    for point in points {
        let serde_json::Value::Object(point) = point else { return Err(()); };
        let Some(serde_json::Value::Number(id)) = point.get("id") else { return Err(()); };
        let id = id.as_u64().ok_or(())? as usize;
        max_id = max_id.max(id as isize);
        let Some(position) = point.get("position") else { return Err(()); };
        let position: Xyz = serde_json::from_value(position.clone()).map_err(|_| ())?;

        let point = Box::new(Point::with_position(
            gl,
            Point3::new(position.x, position.y, position.z),
            Rc::clone(&state.name_repo),
            Rc::clone(shader_manager),
        ));

        entity_manager.add_entity_with_id(point, id);
        state.selector.add_selectable(id);
    }

    for geom in geometry {
        let serde_json::Value::Object(object) = geom else { return Err(()); };
        let Some(serde_json::Value::String(type_)) = object.get("objectType") else { return Err(()); };
        let Some(serde_json::Value::Number(id)) = object.get("id") else { return Err(()); };
        let id = id.as_u64().ok_or(())? as usize;
        max_id = max_id.max(id as isize);

        match type_.as_str() {
            "torus" => {
                torus_from_json(gl, id, state, shader_manager, entity_manager, geom.clone())?
            }
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

        state.selector.add_selectable(id);

        if let Some(serde_json::Value::String(name)) = object.get("name") {
            entity_manager.get_entity_mut(id).set_similar_name(name);
        }
    }

    entity_manager.set_next_id((max_id + 1) as usize);

    Ok(())
}

fn torus_from_json<'gl>(
    gl: &'gl glow::Context,
    id: usize,
    state: &State<'gl, '_>,
    shader_manager: &Rc<ShaderManager<'gl>>,
    entity_manager: &mut EntityManager<'gl>,
    geom: serde_json::Value,
) -> Result<(), ()> {
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
    tref.round_points = jtorus.samples.x as u32;
    tref.tube_points = jtorus.samples.y as u32;
    tref.regenerate_mesh();

    entity_manager.add_entity_with_id(torus, id);
    Ok(())
}

fn bezier_c0_from_json<'gl>(
    gl: &'gl glow::Context,
    id: usize,
    state: &State<'gl, '_>,
    shader_manager: &Rc<ShaderManager<'gl>>,
    entity_manager: &mut EntityManager<'gl>,
    geom: serde_json::Value,
) -> Result<(), ()> {
    let spline: JBezierC0 = serde_json::from_value(geom).map_err(|_| ())?;
    let points: Vec<_> = spline.control_points.iter().map(|p| p.id).collect();
    let spline = Box::new(CubicSplineC0::through_points(
        gl,
        Rc::clone(&state.name_repo),
        Rc::clone(shader_manager),
        points.clone(),
        entity_manager.entities(),
    ));

    entity_manager.add_entity_with_id(spline, id);

    for point in points {
        entity_manager.subscribe(id, point);
    }

    Ok(())
}

fn bezier_c2_from_json<'gl>(
    gl: &'gl glow::Context,
    id: usize,
    state: &State<'gl, '_>,
    shader_manager: &Rc<ShaderManager<'gl>>,
    entity_manager: &mut EntityManager<'gl>,
    geom: serde_json::Value,
) -> Result<(), ()> {
    let spline: JBezierC2 = serde_json::from_value(geom).map_err(|_| ())?;
    let points: Vec<_> = spline.de_boor_points.iter().map(|p| p.id).collect();
    let spline = Box::new(CubicSplineC2::through_points(
        gl,
        Rc::clone(&state.name_repo),
        Rc::clone(shader_manager),
        points.clone(),
        entity_manager.entities(),
    ));

    entity_manager.add_entity_with_id(spline, id);

    for point in points {
        entity_manager.subscribe(id, point);
    }

    Ok(())
}

fn interpolating_from_json<'gl>(
    gl: &'gl glow::Context,
    id: usize,
    state: &State<'gl, '_>,
    shader_manager: &Rc<ShaderManager<'gl>>,
    entity_manager: &mut EntityManager<'gl>,
    geom: serde_json::Value,
) -> Result<(), ()> {
    let spline: JInterpolatedC2 = serde_json::from_value(geom).map_err(|_| ())?;
    let points: Vec<_> = spline.control_points.iter().map(|p| p.id).collect();
    let spline = Box::new(InterpolatingSpline::through_points(
        gl,
        Rc::clone(&state.name_repo),
        Rc::clone(shader_manager),
        points.clone(),
        entity_manager.entities(),
    ));

    entity_manager.add_entity_with_id(spline, id);

    for point in points {
        entity_manager.subscribe(id, point);
    }

    Ok(())
}

fn surface_c0_from_json<'gl>(
    gl: &'gl glow::Context,
    id: usize,
    state: &State<'gl, '_>,
    shader_manager: &Rc<ShaderManager<'gl>>,
    entity_manager: &mut EntityManager<'gl>,
    geom: serde_json::Value,
) -> Result<(), ()> {
    let jsurface: JBezierSurfaceC0 = serde_json::from_value(geom).map_err(|_| ())?;
    let points = jsurface.control_points();

    let mut surface = Box::new(BezierSurfaceC0::new(
        gl,
        Rc::clone(&state.name_repo),
        Rc::clone(shader_manager),
        points.clone(),
        entity_manager.entities(),
        jsurface.args(),
    ));

    let sampling = jsurface.sampling();
    surface.u_patch_divisions = sampling.x as u32;
    surface.v_patch_divisions = sampling.y as u32;

    entity_manager.add_entity_with_id(surface, id);

    for &point in points.iter().flatten() {
        entity_manager.subscribe(id, point);
    }

    Ok(())
}

fn surface_c2_from_json<'gl>(
    gl: &'gl glow::Context,
    id: usize,
    state: &State<'gl, '_>,
    shader_manager: &Rc<ShaderManager<'gl>>,
    entity_manager: &mut EntityManager<'gl>,
    geom: serde_json::Value,
) -> Result<(), ()> {
    let jsurface: JBezierSurfaceC2 = serde_json::from_value(geom).map_err(|_| ())?;
    let points = jsurface.control_points();

    let mut surface = Box::new(BezierSurfaceC2::new(
        gl,
        Rc::clone(&state.name_repo),
        Rc::clone(shader_manager),
        points.clone(),
        entity_manager.entities(),
        jsurface.args(),
    ));

    let sampling = jsurface.sampling();
    surface.u_patch_divisions = sampling.x as u32;
    surface.v_patch_divisions = sampling.y as u32;

    entity_manager.add_entity_with_id(surface, id);

    for &point in points.iter().flatten() {
        entity_manager.subscribe(id, point);
    }

    Ok(())
}

fn camera_json(camera: &mut Camera, json: Option<&serde_json::Value>) -> Result<(), ()> {
    let jcamera: JCamera = match json {
        None => JCamera::new(),
        Some(json) => serde_json::from_value(json.clone()).map_err(|_| ())?,
    };

    camera.set_linear_distance(jcamera.distance);
    camera.center = jcamera.focus_point.point();
    camera.altitude = jcamera.rotation.x.to_radians();
    camera.azimuth = jcamera.rotation.y.to_radians();

    Ok(())
}

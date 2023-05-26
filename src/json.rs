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
    pub object_type: String,
    pub name: String,
    pub id: usize,
    pub patches: Vec<JPatch>,
    #[serde(rename = "parameterWrapped")]
    pub parameter_wrapped: ParameterWrapped,
    pub size: XY,
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
                        let patch_idx: usize = v_patch * self.size.x + u_patch;
                        let point_idx: usize = v * 4 + u;

                        points[u][v] = self.patches[patch_idx].control_points[point_idx].id;
                    }
                }
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

    pub fn sampling(&self) -> XY {
        self.patches[0].samples
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

    pub fn sampling(&self) -> XY {
        self.patches[0].samples
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
            "torus" => {
                let jtorus: JTorus = serde_json::from_value(geom.clone()).map_err(|_| ())?;
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

                torus
            }
            "bezierC0" => {
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

                spline
            }
            "bezierC2" => {
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

                spline
            }
            "interpolatedC2" => {
                let spline: JInterpolatedC2 =
                    serde_json::from_value(geom.clone()).map_err(|_| ())?;
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

                spline
            }
            "bezierSurfaceC0" => {
                let jsurface: JBezierSurfaceC0 =
                    serde_json::from_value(geom.clone()).map_err(|_| ())?;
                let points = jsurface.control_points();

                let mut surface = Box::new(BezierSurfaceC0::new(
                    gl,
                    Rc::clone(&state.name_repo),
                    Rc::clone(&shader_manager),
                    points.clone(),
                    entity_manager.entities(),
                    jsurface.args(),
                ));

                let sampling = jsurface.sampling();
                surface.u_patch_divisions = sampling.x as u32;
                surface.v_patch_divisions = sampling.y as u32;

                for &point in points.iter().flatten() {
                    entity_manager.subscribe(id, point);
                }

                surface
            }
            "bezierSurfaceC2" => {
                let jsurface: JBezierSurfaceC2 =
                    serde_json::from_value(geom.clone()).map_err(|_| ())?;
                let points = jsurface.control_points();

                let mut surface = Box::new(BezierSurfaceC2::new(
                    gl,
                    Rc::clone(&state.name_repo),
                    Rc::clone(&shader_manager),
                    points.clone(),
                    entity_manager.entities(),
                    jsurface.args(),
                ));

                let sampling = jsurface.sampling();
                surface.u_patch_divisions = sampling.x as u32;
                surface.v_patch_divisions = sampling.y as u32;

                for &point in points.iter().flatten() {
                    entity_manager.subscribe(id, point);
                }

                surface
            }
            _ => return Err(()),
        };

        entity_manager.add_entity_with_id(entity, id);
        state.selector.add_selectable(id);
    }

    Ok(())
}

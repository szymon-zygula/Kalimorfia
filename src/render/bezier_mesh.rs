use super::gl_drawable::GlDrawable;
use crate::{math::geometry::bezier::BezierCubicSplineC0, render::opengl, utils};
use glow::HasContext;
use nalgebra::Point3;

#[repr(C)]
struct BezierSegmentInput {
    len: u32,
    points: [Point3<f32>; 4],
}

pub struct BezierMesh<'gl> {
    gl: &'gl glow::Context,
    vertex_buffer: u32,
    vertex_array: u32,
    thickness: f32,
    segment_count: i32,
}

impl<'gl> BezierMesh<'gl> {
    pub fn new(gl: &'gl glow::Context, curve: BezierCubicSplineC0) -> Self {
        let input: Vec<BezierSegmentInput> = curve
            .segments()
            .iter()
            .map(|segment| {
                let mut points: Vec<Point3<f32>> = segment
                    .points()
                    .into_iter()
                    .map(|p| Point3::<f32>::new(p.x as f32, p.y as f32, p.z as f32))
                    .collect();

                let initial_len = points.len();

                while points.len() < 4 {
                    points.push(Point3::origin())
                }

                BezierSegmentInput {
                    points: [points[0], points[1], points[2], points[3]],
                    len: initial_len as u32,
                }
            })
            .collect();

        let raw_input = utils::slice_as_raw(&input);

        let vertex_buffer = unsafe { gl.create_buffer() }.unwrap();
        let vertex_array = opengl::init_vao(gl, || unsafe {
            gl.bind_buffer(glow::ARRAY_BUFFER, Some(vertex_buffer));
            gl.buffer_data_u8_slice(glow::ARRAY_BUFFER, raw_input, glow::STATIC_DRAW);

            gl.vertex_attrib_pointer_i32(
                0,
                1,
                glow::INT,
                std::mem::size_of::<BezierSegmentInput>() as i32,
                0,
            );
            gl.enable_vertex_attrib_array(0);

            Self::vertex_attrib_for_point(gl, 0);
            Self::vertex_attrib_for_point(gl, 1);
            Self::vertex_attrib_for_point(gl, 2);
            Self::vertex_attrib_for_point(gl, 3);
        });

        Self {
            gl,
            vertex_buffer,
            vertex_array,
            thickness: 1.0,
            segment_count: curve.segments().len() as i32,
        }
    }

    unsafe fn vertex_attrib_for_point(gl: &'gl glow::Context, idx: u32) {
        let point_size = std::mem::size_of::<Point3<f32>>() as i32;
        gl.vertex_attrib_pointer_f32(
            idx + 1,
            3,
            glow::FLOAT,
            false,
            std::mem::size_of::<BezierSegmentInput>() as i32,
            std::mem::size_of::<u32>() as i32 + point_size * idx as i32,
        );
        gl.enable_vertex_attrib_array(idx + 1);
    }

    pub fn thickness(&mut self, thickness: f32) {
        self.thickness = thickness;
    }
}

impl<'gl> GlDrawable for BezierMesh<'gl> {
    fn draw(&self) {
        opengl::with_vao(self.gl, self.vertex_array, || unsafe {
            self.gl.line_width(self.thickness);
            self.gl.draw_arrays(glow::POINTS, 0, self.segment_count);
            self.gl.line_width(1.0);
        });
    }
}

impl<'gl> Drop for BezierMesh<'gl> {
    fn drop(&mut self) {
        unsafe {
            self.gl.delete_vertex_array(self.vertex_array);
            self.gl.delete_buffer(self.vertex_buffer);
        }
    }
}

use super::gl_drawable::GlDrawable;
use crate::{
    camera::Camera, math::geometry::bezier::BezierCubicSplineC0, primitives::color::Color,
    render::gl_program::GlProgram, render::opengl, utils,
};
use glow::HasContext;
use nalgebra::{Matrix4, Point3};

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
    const GEOMETRY_SHADER_VERTEX_COUNT: usize = 128;

    pub fn empty(gl: &'gl glow::Context) -> Self {
        let (vertex_array, vertex_buffer) = Self::create_vao_vbo(gl, Vec::new());

        Self {
            gl,
            vertex_buffer,
            vertex_array,
            thickness: 1.0,
            segment_count: 0,
        }
    }

    pub fn new(gl: &'gl glow::Context, curve: BezierCubicSplineC0) -> Self {
        let input = Self::curve_segment_inputs(curve);
        let segment_count = input.len() as i32;

        let (vertex_array, vertex_buffer) = Self::create_vao_vbo(gl, input);

        Self {
            gl,
            vertex_buffer,
            vertex_array,
            thickness: 1.0,
            segment_count,
        }
    }

    fn curve_segment_inputs(curve: BezierCubicSplineC0) -> Vec<BezierSegmentInput> {
        curve
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
            .collect()
    }

    fn create_vao_vbo(gl: &'gl glow::Context, input: Vec<BezierSegmentInput>) -> (u32, u32) {
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

        (vertex_array, vertex_buffer)
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

    pub fn draw_with_program(
        &self,
        program: &GlProgram,
        camera: &Camera,
        segment_pixel_length: f32,
        premul: &Matrix4<f32>,
        color: &Color,
    ) {
        program.enable();
        program.uniform_matrix_4_f32_slice("model", premul.as_slice());
        program.uniform_matrix_4_f32_slice("view", camera.view_transform().as_slice());
        program.uniform_matrix_4_f32_slice("projection", camera.projection_transform().as_slice());
        program.uniform_color("curve_color", color);

        let pass_count = segment_pixel_length as usize / Self::GEOMETRY_SHADER_VERTEX_COUNT * 4 + 1;

        for i in 0..pass_count {
            program.uniform_f32("start", i as f32 / pass_count as f32);
            program.uniform_f32("end", (i + 1) as f32 / pass_count as f32);
            self.draw();
        }
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

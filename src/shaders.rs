use kalimorfia::render::{gl_program::GlProgram, shader::Shader, shader_manager::ShaderManager};
use std::{path::Path, rc::Rc};

const SHADERS_PATH: &str = "shaders/";
const SHADERS_EXTENSION: &str = "glsl";

pub fn create_shader_manager(gl: &glow::Context) -> Rc<ShaderManager> {
    let fragment_colored = shader(gl, "fragment_colored", glow::FRAGMENT_SHADER);
    let fragment_uniform = shader(gl, "uniform_fragment", glow::FRAGMENT_SHADER);
    let fragment_gk = shader(gl, "fragment_gk", glow::FRAGMENT_SHADER);

    let pass_through_vertex = shader(gl, "pass_through_vertex", glow::VERTEX_SHADER);
    let perspective_vertex = shader(gl, "perspective_vertex", glow::VERTEX_SHADER);
    let perspective_vertex_colored = shader(gl, "perspective_vertex_colored", glow::VERTEX_SHADER);
    let perspective_vertex_colored_uniform =
        shader(gl, "perspective_vertex_uniform_color", glow::VERTEX_SHADER);
    let point_cloud_vertex = shader(gl, "point_cloud_vertex", glow::VERTEX_SHADER);
    let vertex_bezier = shader(gl, "vertex_bezier", glow::VERTEX_SHADER);
    let vertex_gk = shader(gl, "vertex_gk", glow::VERTEX_SHADER);

    let geometry_bezier = shader(gl, "geometry_bezier", glow::GEOMETRY_SHADER);

    let surface_tesselation_control =
        shader(gl, "surface_tesselation_control", glow::TESS_CONTROL_SHADER);
    let surface_tesselation_evaluation = shader(
        gl,
        "surface_tesselation_evaluation",
        glow::TESS_EVALUATION_SHADER,
    );

    let gregory_tesselation_control =
        shader(gl, "gregory_tesselation_control", glow::TESS_CONTROL_SHADER);
    let gregory_tesselation_evaluation = shader(
        gl,
        "gregory_tesselation_evaluation",
        glow::TESS_EVALUATION_SHADER,
    );

    let gk_tesselation_control = shader(gl, "gk_tesselation_control", glow::TESS_CONTROL_SHADER);
    let gk_tesselation_evaluation = shader(
        gl,
        "gk_tesselation_evaluation",
        glow::TESS_EVALUATION_SHADER,
    );

    Rc::new(ShaderManager::new(vec![
        (
            "gk_mode",
            GlProgram::with_shaders(
                gl,
                &[
                    &vertex_gk,
                    &gk_tesselation_control,
                    &gk_tesselation_evaluation,
                    &fragment_gk,
                ],
            ),
        ),
        (
            "line_mesh",
            GlProgram::with_shaders(gl, &[&perspective_vertex, &fragment_uniform]),
        ),
        (
            "point",
            GlProgram::with_shaders(gl, &[&point_cloud_vertex, &fragment_colored]),
        ),
        (
            "cursor",
            GlProgram::with_shaders(gl, &[&perspective_vertex_colored, &fragment_colored]),
        ),
        (
            "torus",
            GlProgram::with_shaders(
                gl,
                &[&perspective_vertex_colored_uniform, &fragment_colored],
            ),
        ),
        (
            "spline",
            GlProgram::with_shaders(
                gl,
                &[&perspective_vertex_colored_uniform, &fragment_colored],
            ),
        ),
        (
            "bezier",
            GlProgram::with_shaders(gl, &[&vertex_bezier, &geometry_bezier, &fragment_colored]),
        ),
        (
            "surface",
            GlProgram::with_shaders(
                gl,
                &[
                    &pass_through_vertex,
                    &surface_tesselation_control,
                    &surface_tesselation_evaluation,
                    &fragment_uniform,
                ],
            ),
        ),
        (
            "gregory",
            GlProgram::with_shaders(
                gl,
                &[
                    &pass_through_vertex,
                    &gregory_tesselation_control,
                    &gregory_tesselation_evaluation,
                    &fragment_uniform,
                ],
            ),
        ),
    ]))
}

fn shader<'gl>(gl: &'gl glow::Context, name: &str, kind: u32) -> Shader<'gl> {
    let mut path = Path::new(SHADERS_PATH).join(name);
    path.set_extension(SHADERS_EXTENSION);
    Shader::from_file(gl, &path, kind)
}

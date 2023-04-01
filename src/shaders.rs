use kalimorfia::render::{gl_program::GlProgram, shader::Shader, shader_manager::ShaderManager};
use std::{path::Path, rc::Rc};

const SHADERS_PATH: &str = "shaders/";
const SHADERS_EXTENSION: &str = "glsl";

pub fn create_shader_manager<'gl>(gl: &'gl glow::Context) -> Rc<ShaderManager<'gl>> {
    let simple_fragment = shader(gl, "simple_fragment", glow::FRAGMENT_SHADER);
    let fragment_colored = shader(gl, "fragment_colored", glow::FRAGMENT_SHADER);

    let simple_fragment = shader(gl, "simple_vertex", glow::VERTEX_SHADER);
    let perspective_vertex = shader(gl, "perspective_vertex", glow::VERTEX_SHADER);
    let perspective_vertex_colored = shader(gl, "perspective_vertex_colored", glow::VERTEX_SHADER);
    let perspective_vertex_colored_uniform =
        shader(gl, "perspective_vertex_uniform_color", glow::VERTEX_SHADER);
    let point_cloud_vertex = shader(gl, "point_cloud_vertex", glow::VERTEX_SHADER);

    Rc::new(ShaderManager::new(vec![
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
    ]))
}

fn shader<'gl>(gl: &'gl glow::Context, name: &str, kind: u32) -> Shader<'gl> {
    let mut path = Path::new(SHADERS_PATH).join(name);
    path.set_extension(SHADERS_EXTENSION);
    Shader::from_file(gl, &path, kind)
}

#version 430

layout (location = 0) in vec3 position;
layout (location = 1) in vec2 uv;

uniform mat4 model_transform;
uniform mat4 view_transform;
uniform mat4 projection_transform;

out vec2 trim_coord;

const float pi2 = 2.0 * 3.14159265359;

void main() {
    gl_Position = projection_transform * view_transform * model_transform * vec4(position, 1.0f);
    trim_coord = uv / pi2;
}

#version 430

layout (location = 0) in vec3 position;

out vec3 color;

uniform mat4 model_transform;
uniform mat4 view_transform;
uniform mat4 projection_transform;

uniform float point_size;
uniform vec3 point_color;

void main() {
    gl_Position = projection_transform * view_transform * model_transform * vec4(position, 1.0f);
    gl_PointSize = point_size;
    color = point_color;
}

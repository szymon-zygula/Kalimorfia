#version 430

layout (location = 0) in vec3 position;
layout (location = 1) in vec3 vertex_color;

out vec3 color;

uniform mat4 model_transform;
uniform mat4 view_transform;
uniform mat4 projection_transform;

void main() {
    gl_Position = projection_transform * view_transform * model_transform * vec4(position, 1.0f);
    color = vertex_color;
}

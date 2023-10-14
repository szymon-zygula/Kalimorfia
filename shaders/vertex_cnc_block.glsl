#version 430

layout (location = 0) in vec3 position;
layout (location = 1) in vec3 normal;

out vec3 normal_out;
out vec3 world;

uniform mat4 model_transform;
uniform mat4 view_transform;
uniform mat4 projection_transform;

void main() {
    vec4 norm = transpose(inverse(model_transform)) * vec4(normal, 0.0f);
    normal_out = normalize(norm.xyz);
    world = (model_transform * vec4(position, 1.0f)).xyz;
    gl_Position = projection_transform * view_transform * vec4(world, 1.0f);
}

#version 430

layout (location = 0) in vec3 position;
layout (location = 1) in vec3 normal;
layout (location = 2) in float x;
layout (location = 3) in float y;

layout (binding = 0) uniform sampler2D height_texture;

out vec3 normal_out;
out vec3 world;

uniform mat4 model_transform;
uniform mat4 view_transform;
uniform mat4 projection_transform;

void main() {
    ivec2 texture_size = textureSize(height_texture, 0);
    float x_norm = x / texture_size.x;
    float y_norm = y / texture_size.y;
    float height = 0.0;
    if(!(x_norm < 0.0 || y_norm < 0.0 || x_norm >= 1.0 || y_norm >= 1.0)) {
        height = texture(height_texture, vec2(x_norm, y_norm)).r;
    }

    vec4 norm = transpose(inverse(model_transform)) * vec4(normal, 0.0f);
    normal_out = normalize(norm.xyz);
    world = (model_transform * vec4(position + vec3(0.0, 0.0, height), 1.0f)).xyz;
    gl_Position = projection_transform * view_transform * vec4(world, 1.0f);
}

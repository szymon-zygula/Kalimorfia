#version 430

uniform vec3 cam_pos;
layout(binding = 1) uniform sampler2D color_texture;
layout(binding = 2) uniform sampler2D normal_texture;

in vec3 normal;
in vec3 u_derivative;
in vec3 v_derivative;
in vec3 world;
in float u_glob;
in float v_glob;
in float lod;

out vec4 frag_color;

const vec3 light_pos = vec3(2.5, 5.0, 2.5);

void main() {
    vec3 to_cam = normalize(cam_pos - world);
    vec3 to_light = normalize(light_pos - world);
    vec3 nt = texture(normal_texture, vec2(u_glob, v_glob)).xyz;
    vec3 norm_tex = 2.0 * (nt - vec3(0.5, 0.5, 0.5));
    vec3 norm =
        normalize(norm_tex.y * u_derivative + norm_tex.z * v_derivative + norm_tex.x * normal);

    float ambient = 0.2;
    float diffuse = dot(norm, to_light);
    vec3 reflected = normalize(reflect(-to_light, norm));
    float specular = pow(max(dot(reflected, to_cam), 0.0), 20.0);

    vec3 color = texture(color_texture, vec2(u_glob, v_glob)).xyz;

    frag_color = vec4((ambient + diffuse + specular) * color, 1.0);
}

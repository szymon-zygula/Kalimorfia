#version 430

in vec3 normal_out;
in vec3 world;

out vec4 frag_color;

const vec3 light_pos = vec3(7.0, 7.0, -7.0);
const vec3 color = vec3(0.4, 0.4, 0.8);

uniform vec3 cam_pos;

void main() {
    vec3 to_cam = normalize(cam_pos - world);
    vec3 to_light = normalize(light_pos - world);

    float ambient = 0.3;
    float diffuse =  max(dot(normal_out, to_light), 0.0);
    vec3 reflected = normalize(reflect(-to_light, normal_out));
    float specular = pow(max(dot(reflected, to_cam), 0.0), 50.0);

    frag_color = vec4((ambient + diffuse + specular) * color, 1.0);
}

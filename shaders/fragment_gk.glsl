#version 430

uniform vec3 cam_pos;

in vec3 normal;
in vec3 world;

out vec4 frag_color;

const vec3 light_pos = vec3(2.5, 5.0, 2.5);
const vec3 color = vec3(0.2, 0.4, 0.8);

void main() {
    vec3 to_cam = normalize(cam_pos - world);
    vec3 to_light = normalize(light_pos - world);

    float ambient = 0.2;
    float diffuse = dot(normal, to_light);
    vec3 reflected = normalize(reflect(-to_light, normal));
    float specular = pow(max(dot(reflected, to_cam), 0.0), 20.0);

    frag_color = vec4((ambient + diffuse + specular) * color, 1.0);
}

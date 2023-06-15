#version 430

layout (binding = 0) uniform sampler2D trimmer;
uniform vec3 color;

in vec2 trim_coord;

out vec4 frag_color;

void main() {
    if(texture(trimmer, trim_coord).z > 0.5) {
        frag_color = vec4(color, 1.0f);
    }
    else {
        discard;
    }
}

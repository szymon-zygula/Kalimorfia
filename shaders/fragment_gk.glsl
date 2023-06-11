#version 430

in vec3 normal;

out vec4 frag_color;

void main() {
    frag_color = vec4(normal, 1.0f);
}

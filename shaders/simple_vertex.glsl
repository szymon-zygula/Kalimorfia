#version 430

const vec2 verts[3] = vec2[3](
    vec2(0.5f, -0.5f),
    vec2(0.5f, 0.5f),
    vec2(-0.5f, -0.5f)
);

void main() {
    gl_Position = vec4(verts[gl_VertexID], 0.0f, 1.0);
}

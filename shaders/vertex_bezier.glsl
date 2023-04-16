#version 430

layout (location = 0) in int len;
layout (location = 1) in vec3 pts[4];

out VS_OUT {
    int len;
    vec3 pts[4];
} vs_out;

void main() {
    vs_out.len = len;
    vs_out.pts[0] = pts[0];
    vs_out.pts[1] = pts[1];
    vs_out.pts[2] = pts[2];
    vs_out.pts[3] = pts[3];
}

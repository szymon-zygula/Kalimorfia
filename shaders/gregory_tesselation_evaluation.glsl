#version 430

layout (quads, equal_spacing, ccw) in;

uniform mat4 model;
uniform mat4 view;
uniform mat4 projection;

// Gregory patch structure
// top: [Point3<f32>; 4], 0-3
// top_sides: [Point3<f32>; 2], 4-5
// bottom_sides: [Point3<f32>; 2], 6-7
// bottom: [Point3<f32>; 4], 8-11
// u_inner: [Point3<f32>; 4], 12-15
// v_inner: [Point3<f32>; 4], 16-19

vec3 p(uint idx) {
    return gl_in[idx].gl_Position.xyz;
}

vec3 bezier3(vec3 b0, vec3 b1, vec3 b2, vec3 b3, float t) {
    float t1 = 1.0f - t;

    b0 = t1 * b0 + t * b1;
    b1 = t1 * b1 + t * b2;
    b2 = t1 * b2 + t * b3;

    b0 = t1 * b0 + t * b1;
    b1 = t1 * b1 + t * b2;

    return t1 * b0 + t * b1;
}

const float eps = 1e-10;

vec3 bicubic_bezier(float u, float v) {
    vec3 pi00 = (u * p(12) + v * p(16)) / (u + v + eps);
    vec3 pi01 = (u * p(13) + (1.0 - v) * p(17)) / (u + 1.0 - v + eps);
    vec3 pi10 = ((1.0 - u) * p(14) + v * p(18)) / (1.0 - u + v + eps);
    vec3 pi11 = ((1.0 - u) * p(15) + (1.0 - v) * p(19)) / (2.0 - u - v + eps);

    vec3 p0 = bezier3(p(0), p(1), p(2), p(3), v);
    vec3 p1 = bezier3(p(4), pi00, pi01, p(5), v);
    vec3 p2 = bezier3(p(6), pi10, pi11, p(7), v);
    vec3 p3 = bezier3(p(8), p(9), p(10), p(11), v);

    return bezier3(p0, p1, p2, p3, u);
}

void main() {
    float u = gl_TessCoord.x;
    float v = gl_TessCoord.y;

    vec4 position = vec4(bicubic_bezier(u, v), 1.0f);
    gl_Position = projection * view * model * position;
}

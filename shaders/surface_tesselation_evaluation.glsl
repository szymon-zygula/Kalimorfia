#version 430

layout (quads, equal_spacing, ccw) in;

uniform mat4 model;
uniform mat4 view;
uniform mat4 projection;

// Control point
vec3 p(uint up, uint vp) {
    return gl_in[up * 4 + vp].gl_Position.xyz;
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

vec3 bicubic_bezier(float u, float v) {
    vec3 p0 = bezier3(p(0, 0), p(0, 1), p(0, 2), p(0, 3), v);
    vec3 p1 = bezier3(p(1, 0), p(1, 1), p(1, 2), p(1, 3), v);
    vec3 p2 = bezier3(p(2, 0), p(2, 1), p(2, 2), p(2, 3), v);
    vec3 p3 = bezier3(p(3, 0), p(3, 1), p(3, 2), p(3, 3), v);

    return bezier3(p0, p1, p2, p3, u);
}

void main() {
    float u = gl_TessCoord.x;
    float v = gl_TessCoord.y;

    vec4 position = vec4(bicubic_bezier(u, v), 1.0f);
    gl_Position = projection * view * model * position;
}

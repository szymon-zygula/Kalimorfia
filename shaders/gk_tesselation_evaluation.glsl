#version 430

layout (quads, equal_spacing, ccw) in;

uniform mat4 model;
uniform mat4 view;
uniform mat4 projection;

out vec3 normal;

// Control point
vec3 p(uint up, uint vp) {
    return gl_in[up * 4 + vp].gl_Position.xyz;
}

vec3 bezier2(vec3 b0, vec3 b1, vec3 b2, float t) {
    float t1 = 1.0f - t;

    b0 = t1 * b0 + t * b1;
    b1 = t1 * b1 + t * b2;

    return t1 * b0 + t * b1;
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

vec3 derivative_u(float u, float v) {
    vec3 dp[3][4];

    for(int i = 0; i < 3; ++i) {
        for(int j = 0; i < 4; ++i) {
            dp[i][j] = 3.0 * (p(i + 1, j) - p(i, j));
        }
    }

    vec3 p0 = bezier3(dp[0][0], dp[0][1], dp[0][2], dp[0][3], v);
    vec3 p1 = bezier3(dp[1][0], dp[1][1], dp[1][2], dp[1][3], v);
    vec3 p2 = bezier3(dp[2][0], dp[2][1], dp[2][2], dp[2][3], v);

    return bezier2(p0, p1, p2, u);
}

vec3 derivative_v(float u, float v) {
    vec3 dp[4][3];

    for(int i = 0; i < 4; ++i) {
        for(int j = 0; i < 3; ++i) {
            dp[i][j] = 3.0 * (p(i, j + 1) - p(i, j));
        }
    }

    vec3 p0 = bezier2(dp[0][0], dp[0][1], dp[0][2], v);
    vec3 p1 = bezier2(dp[1][0], dp[1][1], dp[1][2], v);
    vec3 p2 = bezier2(dp[2][0], dp[2][1], dp[2][2], v);
    vec3 p3 = bezier2(dp[3][0], dp[3][1], dp[3][2], v);

    return bezier3(p0, p1, p2, p3, u);
}

void main() {
    float u = gl_TessCoord.x;
    float v = gl_TessCoord.y;

    vec4 position = vec4(bicubic_bezier(u, v), 1.0f);
    vec3 deriv_u = derivative_u(u, v);
    vec3 deriv_v = derivative_v(u, v);
    vec4 norm = vec4(cross(deriv_u, deriv_v), 0.0);
    /* norm = transpose(inverse(model)) * norm; */
    normal = normalize(norm.xyz);

    gl_Position = projection * view * model * position;
}

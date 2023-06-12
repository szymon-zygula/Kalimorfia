#version 430

layout (quads, equal_spacing, ccw) in;

uniform mat4 model;
uniform mat4 view;
uniform mat4 projection;
layout(binding = 0) uniform sampler2D displacement_texture;
uniform uint v_patches;
uniform uint u_patches;
uniform uint subdivisions;

out vec3 normal;
out vec3 u_derivative;
out vec3 v_derivative;
out vec3 world;
out float u_glob;
out float v_glob;
out float lod;

const uint MIN_TESS_LEVEL = 2;
const uint MAX_TESS_LEVEL = 64;

float factor(float dist) {
    return -float(subdivisions) * log(dist * 0.1) / log(10.0);
}

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
        for(int j = 0; j < 4; ++j) {
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
        for(int j = 0; j < 3; ++j) {
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

    u_glob = (gl_PrimitiveID / v_patches + u) / u_patches;
    v_glob = (gl_PrimitiveID % v_patches + v) / v_patches;

    u_derivative = normalize(derivative_u(u, v));
    v_derivative = normalize(derivative_v(u, v));
    vec4 norm = vec4(cross(u_derivative, v_derivative), 0.0);
    norm = transpose(inverse(model)) * norm;
    normal = -normalize(norm.xyz);

    vec3 bez = bicubic_bezier(u, v);
    vec3 view_bez = (view * model * vec4(bez, 1.0)).xyz;
    lod = 6 - log2(max(factor(-view_bez.z / 5.0), 1));
    float displacement =
        textureLod(displacement_texture, vec2(u_glob, v_glob), lod).x;

    vec4 position = vec4(bez + displacement * normal * 0.05, 1.0f);
    world = (model * position).xyz;

    gl_Position = projection * view *vec4(world, 1.0);
}

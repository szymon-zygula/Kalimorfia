#version 430

layout (points) in;
layout (line_strip, max_vertices = 128) out;


uniform mat4 projection;
uniform mat4 view;
uniform mat4 model;
uniform vec3 curve_color;
uniform float start;
uniform float end;

in VS_OUT {
    int len;
    vec3 pts[4];
} gs_in[];

out vec3 color;

const float VERTEX_COUNT = 128;

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

void emit_bezier1() {
    color = curve_color;
    mat4 pvm = projection * view * model;

    gl_Position = pvm * vec4(gs_in[0].pts[0], 1.0f);
    EmitVertex();

    gl_Position = pvm * vec4(gs_in[0].pts[1], 1.0f);
    EmitVertex();
}

void emit_bezier2(float line_length) {
    mat4 pvm = projection * view * model;
    for(int i = 0; i < VERTEX_COUNT; ++i) {
        float t = start + i * line_length;
        gl_Position = pvm * vec4(bezier2(
            gs_in[0].pts[0],
            gs_in[0].pts[1],
            gs_in[0].pts[2],
            t
        ), 1.0f);
        color = curve_color;
        EmitVertex();
    }
}

void emit_bezier3(float line_length) {
    mat4 pvm = projection * view * model;
    for(int i = 0; i < VERTEX_COUNT; ++i) {
        float t = start + i * line_length;
        gl_Position = pvm * vec4(bezier3(
            gs_in[0].pts[0],
            gs_in[0].pts[1],
            gs_in[0].pts[2],
            gs_in[0].pts[3],
            t
        ), 1.0f);
        color = curve_color;
        EmitVertex();
    }
}

void main() {
    float line_length = (end - start) / float(VERTEX_COUNT - 1);

    if(gs_in[0].len == 4) {
        emit_bezier3(line_length);
    }
    else if(gs_in[0].len == 3) {
        emit_bezier2(line_length);
    }
    else if(gs_in[0].len == 2) {
        emit_bezier1();
    }

    EndPrimitive();
}

#version 430

layout (vertices=16) out;

uniform mat4 model;
uniform mat4 view;

const uint MIN_TESS_LEVEL = 2;
const uint MAX_TESS_LEVEL = 32;

uint factor(float dist) {
    float tess = -16.0 * log(dist * 0.05) / log(10.0);
    return uint(round(clamp(tess, MIN_TESS_LEVEL, MAX_TESS_LEVEL)));
}

void main() {
    gl_out[gl_InvocationID].gl_Position = gl_in[gl_InvocationID].gl_Position;

    if(gl_InvocationID == 0)
    {
        vec4 pp00 = gl_in[0].gl_Position;
        vec4 pp01 = gl_in[3].gl_Position;
        vec4 pp10 = gl_in[12].gl_Position;
        vec4 pp11 = gl_in[15].gl_Position;

        vec4 eu0 = (pp00 + pp01) * 0.5;
        vec4 eu1 = (pp10 + pp11) * 0.5;
        vec4 ev0 = (pp00 + pp10) * 0.5;
        vec4 ev1 = (pp01 + pp11) * 0.5;

        float du0 = abs((view * model * eu0).z);
        float du1 = abs((view * model * eu1).z);
        float dv0 = abs((view * model * ev0).z);
        float dv1 = abs((view * model * ev1).z);

        uint fu0 = factor(du0);
        uint fu1 = factor(du1);
        uint fv0 = factor(dv0);
        uint fv1 = factor(dv1);

        gl_TessLevelOuter[0] = fu0;
        gl_TessLevelOuter[1] = fv0;
        gl_TessLevelOuter[2] = fu1;
        gl_TessLevelOuter[3] = fv1;

        float max_tess_level = max(max(max(
            fu0, fv0), fu1), fv1
        );

        gl_TessLevelInner[0] = max_tess_level;
        gl_TessLevelInner[1] = max_tess_level;
    }
}

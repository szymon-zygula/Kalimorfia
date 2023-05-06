#version 430

layout (vertices=16) out;

uniform uint u_subdivisions;
uniform uint v_subdivisions;

void main() {
    gl_out[gl_InvocationID].gl_Position = gl_in[gl_InvocationID].gl_Position;

    if(gl_InvocationID == 0)
    {
        gl_TessLevelOuter[0] = u_subdivisions;
        gl_TessLevelOuter[1] = v_subdivisions;
        gl_TessLevelOuter[2] = u_subdivisions;
        gl_TessLevelOuter[3] = v_subdivisions;

        gl_TessLevelInner[0] = v_subdivisions;
        gl_TessLevelInner[1] = u_subdivisions;
    }
}

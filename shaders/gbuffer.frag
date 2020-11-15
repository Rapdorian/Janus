#version 450

layout(location=0) out vec4 g_pos;
layout(location=1) out vec4 g_norm;
layout(location=2) out vec4 g_col;

layout(location=0) in vec2 a_uv;
layout(location=1) in vec3 l_pos;
layout(location=2) in mat4 mvp;

layout(set=1, binding=0) uniform sampler2D diffuse;

void main() {
    vec3 f_pos = vec3(floor(l_pos.x), floor(l_pos.y), floor(l_pos.z));
    vec4 pos = mvp * vec4(f_pos, 1.0);
    g_pos = pos;
    g_norm = vec4(0.0, 1.0, 1.0, 0.0);
    g_col = texture(diffuse, a_uv);
}

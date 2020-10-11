#version 450

layout(location=0) in vec2 pos;
layout(location=0) out vec2 o_pos;
void main() {
    o_pos = pos;
    gl_Position = vec4(pos, 0.0, 1.0);
}


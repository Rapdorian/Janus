# version 450

layout(location=0) in vec3 pos;
layout(location=1) in vec2 uv;

layout(location=0) out vec2 a_uv;
layout(location=1) out vec3 l_pos;
layout(location=2) out mat4 o_mvp;

layout(set=0, binding=0)
    uniform Uniforms {
        mat4 view_proj;
        mat4 model;
    };

void main() {
    a_uv = uv;
    l_pos = pos;
    o_mvp = model;
    mat4 id = mat4(
            vec4(1.0, 0.0, 0.0, 0.0),
            vec4(0.0, 1.0, 0.0, 0.0),
            vec4(0.0, 0.0, 1.0, 0.0),
            vec4(0.0, 0.0, 0.0, 1.0)
            );
    gl_Position = view_proj * model * vec4(pos, 1.0) ; 
}

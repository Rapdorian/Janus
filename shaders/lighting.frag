#version 450

layout(location=0) out vec4 o_col;

layout(set=0, binding=0) uniform sampler2D g_pos;
layout(set=0, binding=1) uniform sampler2D g_norm;
layout(set=0, binding=2) uniform sampler2D g_col;
layout(location=0) in vec2 a_pos;

void main() {
    //vec2 pos = vec2(clamp(a_pos.x, 0.0, 1.0), clamp(a_pos.y, 0.0, 1.0));
    
    vec3 light_pos = vec3(0.0, 0.0, -20.0);

    vec2 pos = (a_pos + vec2(1.0, 1.0)) / 2.0;
    vec3 col = texture(g_col, pos).rgb;
    vec3 lpos = texture(g_pos, pos).rgb;
    
    float dist = length(lpos - light_pos);
    float power = 90000.0 / (dist * dist * dist);


    power = power / 100;
    //power = clamp(floor(power * 3) / 3, 0.0, 1.0);
    //power = clamp(round(100 * power), 0, 5) / 5;
    //power += 0.001;
    o_col = vec4(col.rgb * power, 1.0);
    //o_col = vec4(dist, dist, dist, 1.0);
    //o_col = vec4((lpos-light_pos) / 20.0, 1.0);
    //o_col = vec4(1.0, 1.0, 1.0, 0.0);
}

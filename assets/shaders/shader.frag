#version 450

layout (location = 0) in vec2 frag_uv;

layout (set = 0, binding = 0) uniform sampler2D tex;

layout (location = 0) out vec4 f_color;

void main() {
    f_color = texture(tex, frag_uv);
}
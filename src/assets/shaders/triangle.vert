#version 450

layout (location = 0) in vec2 position;
layout (location = 1) in vec4 color;

layout (push_constant) uniform PushConstantData {
    mat2 transform;
    vec2 offset;
} push;

layout (location = 0) out vec4 o_color;

void main() {
    gl_Position = vec4(push.transform * position + push.offset, 0.0, 1.0);
    o_color = color;
}
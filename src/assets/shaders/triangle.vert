#version 450

layout (location = 0) in vec3 position;
layout (location = 1) in vec4 color;

layout (push_constant) uniform PushConstantData {
    mat4 transform;
} push;

layout (location = 0) out vec4 o_color;

void main() {
    gl_Position = push.transform * vec4(position, 1.0);
    o_color = color;
}
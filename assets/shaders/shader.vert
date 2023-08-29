#version 450

layout (location = 0) in vec3 position;
layout (location = 1) in vec4 color;
layout (location = 2) in vec2 uv;
layout (location = 3) in vec3 normal;

layout (push_constant) uniform PushConstantData {
    mat4 transform;
} push;

layout (location = 0) out vec2 frag_uv;

void main() {
    gl_Position = push.transform * vec4(position, 1.0);
    frag_uv = uv;
}
#version 450

#extension GL_GOOGLE_include_directive : require

#include "../include/defs.glsl"

layout(local_size_x_id = 0, local_size_y = 1, local_size_z = 1) in;

layout(set = 0, binding = 0) readonly uniform Info {
    uvec2 screen;
};
layout(std140, set = 0, binding = 1) readonly uniform Camera {
    vec3 pos;
    mat3 rotation;
    float fov;
};

layout(set = 1, binding = 0) readonly buffer InputBuffer {
    uvec4 colors[];
};

layout(set = 2, binding = 0, rgba8) writeonly uniform image2D resultImage;

void main() {
    uint idx = gl_GlobalInvocationID.x;
    ivec2 pixel_pos = ivec2(idx % screen.x, idx / screen.x);

    uvec4 input_color = colors[idx];

    if (input_color.a == 0) {
        return;
    }

    vec4 out_color = vec4(input_color) / (255 * 255 * 255);

    imageStore(resultImage, pixel_pos, out_color);
}
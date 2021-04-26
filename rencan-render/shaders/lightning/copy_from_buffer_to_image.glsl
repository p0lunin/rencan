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

layout(set = 2, binding = 0, rgba8) uniform image2D resultImage;

layout(push_constant) readonly uniform Offsets {
    uint offset;
} offsets;

void main() {
    uint idx = gl_GlobalInvocationID.x;
    ivec2 pixel_pos = ivec2((offsets.offset + idx) % (screen.x * 2), (offsets.offset + idx) / (screen.x * 2));

    uvec4 input_color = colors[idx];

    if (input_color.a == 0) {
        return;
    }

    vec4 out_color = vec4(input_color) / (255 * 255 * 255);

    vec4 this_color = imageLoad(resultImage, pixel_pos);

    vec4 res_color = this_color + out_color;
    imageStore(resultImage, pixel_pos, res_color);
}

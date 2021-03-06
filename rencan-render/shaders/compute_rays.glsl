#version 450

#extension GL_GOOGLE_include_directive : require

#include "include/defs.glsl"

layout(local_size_x_id = 0, local_size_y = 1, local_size_z = 1) in;

layout(set = 0, binding = 0) readonly uniform Info {
    uvec2 screen;
};
layout(std140, set = 0, binding = 1) readonly uniform Camera {
    vec3 pos;
    mat3 rotation;
    float fov;
};
layout(std140, set = 0, binding = 2) writeonly buffer Rays {
    Ray rays[];
};
layout(std140, set = 0, binding = 3) readonly buffer Intersections {
    Intersection intersections[];
};
layout(set = 0, binding = 4, rgba8) readonly uniform image2D resultImage;

uint compute_x(uint screen_width) {
    return gl_GlobalInvocationID.x % screen_width;
}

uint compute_y(uint screen_width) {
    return gl_GlobalInvocationID.x / screen_width;
}

void main() {
    uint screen_width = screen.x;
    uint screen_height = screen.y;

    uint idx = gl_GlobalInvocationID.x;

    vec2 this_point = vec2(compute_x(screen_width), compute_y(screen_width));

    float scale = tan(fov / 2);
    float aspect_ratio = float(screen_width) / float(screen_height);

    vec3 origin = pos;

    float x = (2 * ((this_point.x + 0.5) / float(screen_width)) - 1) * aspect_ratio * scale;
    float y = (1 - 2 * ((this_point.y + 0.5) / float(screen_height))) * scale;

    vec4 direction = vec4(normalize(rotation * vec3(x, y, -1.0)), 0.0);

    rays[idx] = Ray(origin, direction, 1.0 / 0.0);
}
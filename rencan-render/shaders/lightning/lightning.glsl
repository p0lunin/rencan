#version 450

#extension GL_GOOGLE_include_directive : require

#include "../include/defs.glsl"

const uint SPECULAR_EXPONENT = 200;

layout(local_size_x_id = 0, local_size_y = 1, local_size_z = 1) in;

layout(set = 0, binding = 0) readonly uniform Info {
    uvec2 screen;
};
layout(std140, set = 0, binding = 1) readonly uniform Camera {
    vec3 pos;
    mat3 rotation;
    float fov;
};

layout(std140, set = 1, binding = 0) readonly buffer Intersections {
    LightRay intersections[];
};

layout(set = 2, binding = 0) writeonly buffer ResultImage {
    uvec4 colors[];
};

layout(std140, set = 3, binding = 0) readonly buffer PreviousIntersections {
    Intersection previous_intersections[];
};

void main() {
    uint idx = gl_GlobalInvocationID.x;

    LightRay light_int = intersections[idx];
    Intersection inter = previous_intersections[light_int.inter_id];

    vec3 color = compute_light_color(
        inter.model_material,
        light_int.light_intensity,
        inter.normal,
        light_int.ray.direction,
        inter.ray.direction
    );

    uvec3 add_color = uvec3(color * (255 * 255 * 255));
    atomicAdd(colors[inter.pixel_id].x, add_color.x);
    atomicAdd(colors[inter.pixel_id].y, add_color.y);
    atomicAdd(colors[inter.pixel_id].z, add_color.z);
    atomicAdd(colors[inter.pixel_id].w, (255 * 255 * 255));
}

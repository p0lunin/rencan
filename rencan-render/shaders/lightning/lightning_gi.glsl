#version 450

#extension GL_GOOGLE_include_directive : require

#include "../include/defs.glsl"

layout(local_size_x_id = 0, local_size_y = 1, local_size_z = 1) in;

layout(constant_id = 1) const uint SAMPLES_PER_BOUNCE = 64;

layout(std140, set = 0, binding = 0) readonly buffer Intersections {
    LightRay intersections[];
};

layout(set = 1, binding = 0) writeonly buffer ResultImage {
    uvec4 colors[];
};

layout(std140, set = 2, binding = 0) readonly buffer PreviousIntersections {
    Intersection previous_intersections[];
};

layout(set = 3, binding = 0) readonly buffer GiThethas {
    float gi_thethas[];
};

void main() {
    uint idx = gl_GlobalInvocationID.x;

    LightRay light_int = intersections[idx];
    Intersection inter = previous_intersections[light_int.inter_id];
    if (inter.is_intersect != 1) {
        return;
    }

    float theta = gi_thethas[light_int.inter_id];
    vec3 color = theta * compute_light_color(
        inter.model,
        light_int.light_intensity,
        inter.normal,
        light_int.ray.direction,
        inter.ray.direction
    ) / (1 / (2 * PI)) / SAMPLES_PER_BOUNCE;

    uvec4 add_color = uvec4(clamp(color, 0, 1) * (255 * 255 * 255), (255 * 255 * 255));
    atomicAdd(colors[inter.pixel_id].x, add_color.x);
    atomicAdd(colors[inter.pixel_id].y, add_color.y);
    atomicAdd(colors[inter.pixel_id].z, add_color.z);
    atomicAdd(colors[inter.pixel_id].w, add_color.w);
}

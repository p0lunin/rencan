#version 450

#extension GL_GOOGLE_include_directive : require

layout(constant_id = 1) const uint MAX_INDIRECT_RAYS = 32;

layout(local_size_x_id = 0, local_size_y = 1, local_size_z = 1) in;

#include "include/defs.glsl"

layout(set = 0, binding = 0) readonly buffer Rays {
    LightningRay rays[];
};
layout(set = 0, binding = 1) writeonly buffer Intersections {
    Intersection intersections[];
};

layout(std140, set = 1, binding = 0) readonly uniform SceneInfo {
    uint model_counts;
};
layout(std140, set = 1, binding = 1) readonly buffer ModelInfos {
    ModelInfo models[];
};
layout(set = 1, binding = 2) readonly buffer Vertices {
    vec3[] vertices;
};
layout(std140, set = 1, binding = 3) readonly buffer Indexes {
    uvec3[] indexes;
};
layout(std140, set = 1, binding = 4) readonly buffer HitBoxes {
    HitBoxRectangle[] hit_boxes;
};

#include "include/ray_tracing.glsl"

void main() {
    uint idx = gl_GlobalInvocationID.x;

    uint offset = idx * MAX_INDIRECT_RAYS;

    for (int i = 0; i < MAX_INDIRECT_RAYS; i++) {
        LightningRay ray = rays[offset + i];
        if (ray.ray_type == RAY_TYPE_SHADING) {
            Intersection inter = trace_first(ray.ray);
            intersections[offset + i] = inter;
        }
    }
}
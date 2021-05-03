#version 450

#extension GL_GOOGLE_include_directive : require
#extension GL_EXT_shader_atomic_int64 : require

layout(local_size_x_id = 0, local_size_y = 1, local_size_z = 1) in;

#include "include/defs.glsl"

layout(std140, set = 0, binding = 0) readonly buffer Rays {
    Ray rays[];
};

layout(std140, set = 1, binding = 0) writeonly buffer Intersections {
    Intersection intersections[];
};

layout(std140, set = 2, binding = 0) readonly uniform SceneInfo {
    uint model_counts;
};
layout(std140, set = 2, binding = 1) readonly buffer ModelInfos {
    ModelInfo models[];
};
layout(set = 2, binding = 2) readonly buffer Vertices {
    vec3[] vertices;
};
layout(std140, set = 2, binding = 3) readonly buffer Indexes {
    uvec3[] indexes;
};
layout(std140, set = 2, binding = 4) readonly buffer HitBoxes {
    HitBoxRectangle[] hit_boxes;
};

layout(std140, set = 3, binding = 0) readonly buffer SphereModelsInfo {
    uint sphere_models_count;
};
layout(std140, set = 3, binding = 1) readonly buffer SphereModels {
    ModelInfo sphere_models[];
};
layout(std140, set = 3, binding = 2) readonly buffer Spheres {
    Sphere[] spheres;
};

layout(set = 4, binding = 0) writeonly buffer IntersectionsCount {
    uint count_intersections;
    uint _y_dimension;
    uint _z_dimension;
};

#include "include/ray_tracing.glsl"

void main() {
    uint idx = gl_GlobalInvocationID.x;

    Ray ray = rays[idx];
    Intersection inter;
    bool is_inter = trace(ray, idx, inter);

    if (is_inter) {
        uint intersection_idx = atomicAdd(count_intersections, 1);
        intersections[intersection_idx] = inter;
    }
}
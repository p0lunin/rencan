#version 450

#extension GL_GOOGLE_include_directive : require
#extension GL_EXT_shader_atomic_int64 : require

const uint MAX_DEPTH = 100;

layout(local_size_x_id = 0, local_size_y = 1, local_size_z = 1) in;

#include "../include/defs.glsl"

layout(std140, set = 0, binding = 0) readonly uniform SceneInfo {
    uint model_counts;
};
layout(std140, set = 0, binding = 1) readonly buffer ModelInfos {
    ModelInfo models[];
};
layout(set = 0, binding = 2) readonly buffer Vertices {
    vec3[] vertices;
};
layout(std140, set = 0, binding = 3) readonly buffer Indexes {
    uvec3[] indexes;
};
layout(std140, set = 0, binding = 4) readonly buffer HitBoxes {
    HitBoxRectangle[] hit_boxes;
};

layout(std140, set = 1, binding = 0) readonly buffer SphereModelsInfo {
    uint sphere_models_count;
};
layout(std140, set = 1, binding = 1) readonly buffer SphereModels {
    ModelInfo sphere_models[];
};
layout(std140, set = 1, binding = 2) readonly buffer Spheres {
    Sphere[] spheres;
};

layout(std140, set = 2, binding = 0) restrict buffer Intersections {
    Intersection previous_intersections[];
};

#include "../include/ray_tracing.glsl"

void main() {
    uint idx = gl_GlobalInvocationID.x;

    Intersection inter = previous_intersections[idx];
    bool is_inter = true;
    uint depth = 0;

    vec3 next_direction;
    Ray reflect_ray;

    while (is_inter && inter.model_material.material == MATERIAL_MIRROR && depth < MAX_DEPTH) {
        depth += 1;
        next_direction = reflect(inter.ray.direction, inter.normal);
        reflect_ray = Ray(inter.point, next_direction, 1.0 / 0.0);

        is_inter = trace(reflect_ray, inter.pixel_id, inter);
    }

    if (is_inter && inter.model_material.material != MATERIAL_MIRROR && depth >= 1) {
        previous_intersections[idx] = inter;
    }
}
#version 450

#extension GL_GOOGLE_include_directive : require

layout(local_size_x = 64, local_size_y = 1, local_size_z = 1) in;

#include "include/defs.glsl"

layout(set = 0, binding = 0) readonly uniform Info {
    uvec2 screen;
};
layout(std140, set = 0, binding = 1) readonly uniform Camera {
    vec3 pos;
    mat3 rotation;
    float fov;
};
layout(std140, set = 0, binding = 2) readonly buffer Rays {
    Ray rays[];
};
layout(std140, set = 0, binding = 3) writeonly buffer Intersections {
    Intersection intersections[];
};
layout(set = 0, binding = 4, rgba8) writeonly uniform image2D resultImage;

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

    Ray ray = rays[idx];
    Intersection inter = trace(ray);

    intersections[idx] = inter;
}
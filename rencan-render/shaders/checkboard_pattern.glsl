#version 450

#extension GL_GOOGLE_include_directive : require

#include "include/defs.glsl"

layout(local_size_x = 64, local_size_y = 1, local_size_z = 1) in;

layout(constant_id = 0) const float CHESSBOARD_SCALE = 1.0;

layout(set = 0, binding = 0) readonly uniform Info {
    uvec2 screen;
};
layout(std140, set = 0, binding = 1) readonly uniform Camera {
    vec3 pos;
    mat3 rotation;
    float fov;
};
layout(std140, set = 1, binding = 0) readonly buffer Rays {
    Ray rays[];
};
layout(std140, set = 1, binding = 1) readonly buffer Intersections {
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

layout(set = 3, binding = 0, rgba8) writeonly uniform image2D resultImage;

void main() {
    uint idx = gl_GlobalInvocationID.x;

    Intersection inter = intersections[idx];

    ivec2 pos = ivec2(idx % screen.x, idx / screen.x);

    if (inter.is_intersect == 1) {
        vec3 local_coords = inter.point;

        local_coords = local_coords / CHESSBOARD_SCALE;

        float chessboard = fract((floor(local_coords.x) + floor(local_coords.y) + floor(local_coords.z)) * 0.5);
        chessboard = chessboard * 2;

        imageStore(resultImage, pos, vec4(chessboard));
    }
    else if (inter.is_intersect == 0) {
        imageStore(resultImage, pos, vec4(0.3, 0.4, 0.7, 0.0));
    }
}
#version 450

#extension GL_GOOGLE_include_directive : require

#include "defs.glsl"

layout(local_size_x = 64, local_size_y = 1, local_size_z = 1) in;

layout(constant_id = 0) const float CHESSBOARD_SCALE = 1.0;

layout(set = 0, binding = 0) uniform Info {
    uvec2 screen;
};
layout(std140, set = 0, binding = 1) uniform Camera {
    mat4 cameraToWorld;
    float fov;
};
layout(std140, set = 0, binding = 2) buffer Rays {
    Ray rays[];
};
layout(set = 0, binding = 3, rgba8) uniform image2D resultImage;
layout(std140, set = 0, binding = 4) buffer Intersections {
    Intersection intersections[];
};
layout(std140, set = 0, binding = 5) buffer DirectLightInfo {
    DirectLight global_light;
};
layout(std140, set = 1, binding = 0) buffer ModelInfo {
    mat4 isometry;
    uint model_id;
    uint indexes_length;
    float albedo;
};
layout(set = 1, binding = 1) buffer Vertices {
    vec3[] vertices;
};
layout(std140, set = 1, binding = 2) buffer Indexes {
    uvec3[] indexes;
};

const float eps = 0.001;

void main() {
    uint idx = gl_GlobalInvocationID.x;

    Intersection inter = intersections[idx];

    ivec2 pos = ivec2(idx % screen.x, idx / screen.x);

    if (inter.is_intersect == 1 && inter.model_id == model_id) {
        uvec3 index = indexes[inter.triangle_idx];

        vec3 local_coords =
            vertices[index.y] * inter.barycentric_coords.x +
            vertices[index.z] * inter.barycentric_coords.y +
            vertices[index.x] * (1 - inter.barycentric_coords.x - inter.barycentric_coords.y);

        local_coords = local_coords / CHESSBOARD_SCALE;

        float chessboard = fract((floor(local_coords.x) + floor(local_coords.y) + floor(local_coords.z)) * 0.5);
        chessboard = chessboard * 2;

        imageStore(resultImage, pos, vec4(chessboard));
    }
    else if (inter.is_intersect == 0) {
        imageStore(resultImage, pos, vec4(0.3, 0.4, 0.7, 0.0));
    }
}
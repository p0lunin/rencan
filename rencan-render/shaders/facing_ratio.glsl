#version 450

#extension GL_GOOGLE_include_directive : require

#include "defs.glsl"

layout(local_size_x = 64, local_size_y = 1, local_size_z = 1) in;

layout(set = 0, binding = 0) uniform Info {
    uvec2 screen;
};
layout(std140, set = 0, binding = 1) buffer Rays {
    Ray rays[];
};
layout(set = 0, binding = 2, rgba8) uniform image2D resultImage;
layout(std140, set = 0, binding = 3) buffer Intersections {
    Intersection intersections[];
};
layout(std140, set = 1, binding = 0) buffer ModelInfos {
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

        vec3 normal = normalize(cross(vertices[index.y] - vertices[index.x], vertices[index.z] - vertices[index.x]));

        vec3 ray_direction = normalize((inverse(mat3(isometry[0].xyz, isometry[1].xyz, isometry[2].xyz))) * rays[idx].direction.xyz);

        vec3 color = vec3(max(dot(normal, -ray_direction), 0.0));

        imageStore(resultImage, pos, vec4(color, 0.0));
    }
    else if (inter.is_intersect == 0) {
        imageStore(resultImage, pos, vec4(0.0, 0.4, 0.7, 0.0));
    }
}
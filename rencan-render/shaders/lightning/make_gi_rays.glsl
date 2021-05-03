#version 450

#extension GL_GOOGLE_include_directive : require
#extension GL_EXT_shader_atomic_int64 : require

layout(local_size_x_id = 0, local_size_y = 1, local_size_z = 1) in;

#include "../include/defs.glsl"

layout(std140, set = 0, binding = 0) restrict writeonly buffer Rays {
    Intersection gi_intersects[];
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

layout(std140, set = 2, binding = 0) readonly buffer SphereModelsInfo {
    uint sphere_models_count;
};
layout(std140, set = 2, binding = 1) readonly buffer SphereModels {
    ModelInfo sphere_models[];
};
layout(std140, set = 2, binding = 2) readonly buffer Spheres {
    Sphere[] spheres;
};

layout(set = 3, binding = 0) restrict writeonly buffer IntersectionsCount {
    uint count_intersections;
    uint _y_dimension;
    uint _z_dimension;
};

layout(std140, set = 4, binding = 0) restrict readonly buffer Intersections {
    Intersection previous_intersections[];
};

layout(set = 5, binding = 0) restrict writeonly buffer GiThetas {
    float gi_ethas[];
};

layout(set = 6, binding = 0) restrict readonly uniform Info {
    uvec2 screen;
};
layout(std140, set = 6, binding = 1) restrict readonly uniform Camera {
    vec3 pos;
    mat3 rotation;
    float fov;
};

layout(push_constant) readonly uniform RandomSeed {
    float val1;
    float val2;
    uint offset;
    uint msaa;
} random;

#include "../include/ray_tracing.glsl"

vec3 uniform_sample_hemisphere(float r1, float r2) {
    float sinTheta = sqrt(1 - r1 * r1);
    float phi = 2 * PI * r2;
    vec2 xz = sinTheta * vec2(cos(phi), sin(phi));
    vec3 direction = vec3(xz.x, r1, xz.y);

    return direction;
}
/*
vec3 uniform_sample_hemisphere(float r1, float r2) {
    vec3 direction = vec3(
        sqrt(r1) * cos(2 * PI * r2),
        sqrt(1 - r1),
        sqrt(r1) * sin(2 * PI * r2)
    );

    return direction;
}*/

mat3 create_coordinate_system(vec3 normal) {
    vec3 normal_x;
    float deleter;
    if (abs(normal.x) > abs(normal.y)) {
        normal_x = vec3(normal.z, 0, -normal.x);
        deleter = sqrt(normal.x * normal.x + normal.z * normal.z);
    }
    else {
        normal_x = vec3(0, -normal.z, normal.y);
        deleter = sqrt(normal.y * normal.y + normal.z * normal.z);
    }
    normal_x /= deleter;
    vec3 normal_y = cross(normal, normal_x);
    return mat3(normal_x, normal, normal_y);
}

uint next_idx() {
    return atomicAdd(count_intersections, 1);
}

void main() {
    uint idx = gl_GlobalInvocationID.x;

    Intersection inter = previous_intersections[idx];
    ivec2 pixel_pos = ivec2(
        (random.offset + idx) % (screen.x * random.msaa),
        (random.offset + idx) / (screen.x * random.msaa)
    );

    float r1;
    float r2;

    r1 = rand(pixel_pos * random.val1);
    r2 = rand(pixel_pos * random.val2);

    vec3 next_ray_direction = uniform_sample_hemisphere(r1, r2);
    mat3 transf = create_coordinate_system(inter.normal);

    vec3 next_ray_direction_global = transf * next_ray_direction;

    Ray next_ray = Ray(inter.point, next_ray_direction_global, 1.0 / 0.0);

    Intersection next_inter;
    bool is_inter = trace(next_ray, inter.pixel_id, next_inter);
    if (is_inter) {
        uint idx = next_idx();
        gi_ethas[idx] = r1;
        gi_intersects[idx] = next_inter;
    }
}
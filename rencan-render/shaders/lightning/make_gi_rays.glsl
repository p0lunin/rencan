#version 450

#extension GL_GOOGLE_include_directive : require
#extension GL_EXT_shader_atomic_int64 : require

layout(local_size_x_id = 0, local_size_y = 1, local_size_z = 1) in;

layout(constant_id = 1) const uint SAMPLES_PER_BOUNCE = 64;

#include "../include/defs.glsl"

layout(std140, set = 0, binding = 0) writeonly buffer Rays {
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

layout(set = 3, binding = 0) writeonly buffer IntersectionsCount {
    uint count_intersections;
    uint _y_dimension;
    uint _z_dimension;
};

layout(std140, set = 4, binding = 0) readonly buffer Intersections {
    Intersection previous_intersections[];
};

layout(std140, set = 5, binding = 0) writeonly buffer GiThetas {
    float gi_ethas[];
};

layout(push_constant) uniform RandomSeed {
    float val1;
    float val2;
} random;

#include "../include/ray_tracing.glsl"

vec3 uniform_sample_hemisphere(float r1, float r2) {
    // cos(theta) = r1 = y
    // cos^2(theta) + sin^2(theta) = 1 -> sin(theta) = srtf(1 - cos^2(theta))
    float sinTheta = sqrt(1 - r1 * r1);
    float phi = 2 * PI * r2;
    float x = sinTheta * cos(phi);
    float z = sinTheta * sin(phi);
    return vec3(x, r1, z);
}

mat3 create_coordinate_system(vec3 normal) {
    vec3 normal_x;
    if (abs(normal.x) > abs(normal.y))
        normal_x = vec3(normal.z, 0, -normal.x) / sqrt(normal.x * normal.x + normal.z * normal.z);
    else
        normal_x = vec3(0, -normal.z, normal.y) / sqrt(normal.y * normal.y + normal.z * normal.z);
    vec3 normal_y = cross(normal, normal_x);
    return mat3(normal_x, normal, normal_y);
}

uint next_idx() {
    return atomicAdd(count_intersections, 1);
}

void main() {
    uint idx = gl_GlobalInvocationID.x;

    Intersection inter = previous_intersections[idx];

    if (inter.model.material == MATERIAL_DIFFUSE) {
        float r1;
        float r2 = inter.distance * length(inter.point);

        vec3 indirect_color = vec3(0.0);

        r1 = rand(vec2(random.val1, random.val2));
        r2 = rand(vec2(r1, r2));
        vec3 next_ray_direction = uniform_sample_hemisphere(r1, r2);
        mat3 transf = create_coordinate_system(inter.normal);

        vec3 next_ray_direction_global = transf * next_ray_direction;

        Ray next_ray = Ray(inter.point, next_ray_direction_global, 1.0 / 0.0);

        Intersection next_inter = trace(next_ray, inter.pixel_id);
        if (next_inter.is_intersect == 1.0) {
            uint idx = next_idx();
            gi_intersects[next_idx()] = next_inter;
            gi_ethas[idx] = r1;
        }
    }
}
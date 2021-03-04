#version 450

#extension GL_GOOGLE_include_directive : require

#include "defs.glsl"

layout(local_size_x = 64, local_size_y = 1, local_size_z = 1) in;

layout(set = 0, binding = 0) readonly uniform Info {
    uvec2 screen;
};
layout(std140, set = 0, binding = 1) readonly buffer PrimaryRays {
    Ray primary_rays[];
};
layout(std140, set = 0, binding = 2) readonly buffer ShadowRays {
    Ray shadow_rays[];
};
layout(set = 0, binding = 3, rgba8) writeonly uniform image2D resultImage;
layout(std140, set = 0, binding = 4) readonly buffer PrimaryIntersections {
    Intersection primary_rays_intersections[];
};
layout(std140, set = 0, binding = 5) readonly buffer ShadowIntersections {
    Intersection shadow_rays_intersections[];
};
layout(std140, set = 0, binding = 6) readonly uniform DirectLightInfo {
    DirectLight global_light;
};
layout(std140, set = 0, binding = 7) readonly uniform PointLightsInfo {
    uint point_lights_count;
};
layout(std140, set = 0, binding = 8) readonly buffer PointLights {
    PointLight[] point_lights;
};
layout(std140, set = 1, binding = 0) readonly buffer ModelInfos {
    ModelInfo[] models;
};
layout(set = 1, binding = 1) readonly buffer Vertices {
    vec3[] vertices;
};
layout(std140, set = 1, binding = 2) readonly buffer Indexes {
    uvec3[] indexes;
};

const float eps = 0.001;

#define PI radians(180)

vec3 compute_color_for_global_lights(
    vec3 normal,
    Intersection inter,
    Intersection shadow_intersect,
    ModelInfo model,
    Ray primary_ray
) {
    mat4 isometry = model.isometry;
    float albedo = model.albedo;

    vec3 light_dir = -global_light.direction;

    vec3 ray_direction = normalize(
        (inverse(
            mat3(isometry[0].xyz, isometry[1].xyz, isometry[2].xyz))) * primary_ray.direction.xyz
    );

    float visibility = shadow_intersect.is_intersect == 1 ? 0.0 : 1.0;

    vec3 color = visibility * albedo / PI * global_light.intensity * global_light.color * max(dot(normal, -global_light.direction.xyz), 0.0);

    return color;
}

vec3 compute_color_for_point_light(
    vec3 normal,
    vec3 light_dir,
    PointLight light,
    float albedo,
    float distance
) {
    vec3 intensity = light.intensity * light.color / (4 * PI * distance);

    vec3 color = albedo / PI * intensity * max(dot(normal, -light_dir), 0.0);
    return intensity;
}

void main() {
    uint idx = gl_GlobalInvocationID.x;

    Intersection inter = primary_rays_intersections[idx];
    Ray primary_ray = primary_rays[idx];

    ivec2 pos = ivec2(idx % screen.x, idx / screen.x);

    if (inter.is_intersect == 1) {
        ModelInfo model = models[inter.model_id];
        uvec3 index = indexes[inter.triangle_idx];
        vec3 normal = inter.normal;

        vec3 color = compute_color_for_global_lights(
            normal,
            inter,
            shadow_rays_intersections[idx],
            model,
            primary_ray
        );

        for (int i = 0; i < point_lights_count; i++) {
            uint offset = (i + 1) * screen.x * screen.y;
            PointLight light = point_lights[i];
            vec3 light_dir = light.position - inter.point;
            float distance_to_light = length(light_dir);

            Intersection shadow_intersection = shadow_rays_intersections[offset + idx];
            if (shadow_intersection.is_intersect == 1) {
                continue;
            }
            color = color + compute_color_for_point_light(
                normal,
                light_dir,
                light,
                model.albedo,
                distance_to_light
            );
        }

        imageStore(resultImage, pos, vec4(color, 0.0));
    }
    else if (inter.is_intersect == 0) {
        imageStore(resultImage, pos, vec4(0.0, 0.7, 0.4, 0.0));
    }
}
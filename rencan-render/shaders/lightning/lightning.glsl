#version 450

#extension GL_GOOGLE_include_directive : require

#include "include/defs.glsl"

layout(local_size_x_id = 0, local_size_y = 1, local_size_z = 1) in;

layout(constant_id = 1) const uint MAX_INDIRECT_RAYS = 32;

layout(set = 0, binding = 0) readonly uniform Info {
    uvec2 screen;
};
layout(std140, set = 0, binding = 1) readonly uniform Camera {
    vec3 pos;
    mat3 rotation;
    float fov;
};

layout(set = 1, binding = 0) readonly buffer Rays {
    Ray primary_rays[];
};
layout(set = 1, binding = 1) readonly buffer Intersections {
    Intersection primary_rays_intersections[];
};

layout(set = 2, binding = 0) readonly buffer IndirectRays {
    LightningRay indirect_rays[];
};
layout(set = 2, binding = 1) readonly buffer IndirectIntersections {
    Intersection indirect_intersections[];
};

layout(std140, set = 3, binding = 0) readonly uniform SceneInfo {
    uint model_counts;
};
layout(std140, set = 3, binding = 1) readonly buffer ModelInfos {
    ModelInfo[] models;
};
layout(set = 3, binding = 2) readonly buffer Vertices {
    vec3[] vertices;
};
layout(std140, set = 3, binding = 3) readonly buffer Indexes {
    uvec3[] indexes;
};
layout(std140, set = 3, binding = 4) readonly buffer HitBoxes {
    HitBoxRectangle[] hit_boxes;
};

layout(std140, set = 4, binding = 0) readonly uniform DirectLightInfo {
    DirectLight global_light;
};
layout(std140, set = 4, binding = 1) readonly uniform PointLightsInfo {
    uint point_lights_count;
};
layout(std140, set = 4, binding = 2) readonly buffer PointLights {
    PointLight[] point_lights;
};

layout(set = 5, binding = 0, rgba8) writeonly uniform image2D resultImage;

#define PI radians(180)

vec3 compute_color_for_global_lights(
    Intersection inter,
    Intersection shadow_intersect,
    ModelInfo model,
    Ray primary_ray
) {
    mat4 isometry = model.isometry;
    float albedo = model.albedo;

    vec3 light_dir = -global_light.direction;

    vec3 ray_direction = (inverse(isometry) * primary_ray.direction).xyz;

    vec3 color = albedo / PI * global_light.intensity * global_light.color * max(dot(inter.normal, -global_light.direction.xyz), 0.0);

    return color;
}

vec3 compute_color_for_point_light(
    vec3 normal,
    vec3 light_dir,
    PointLight light,
    float albedo,
    float distance
) {
    vec3 intensity = light.intensity * light.color / (4 * PI * distance * distance);

    vec3 color = albedo / PI * intensity * max(dot(normal, -light_dir), 0.0);
    return intensity;
}

vec3 compute_color_diffuse_material(uint rays_offset, ModelInfo model, Intersection inter, Ray primary_ray) {
    vec3 color = vec3(0.0);

    Intersection direction_light_intersection = indirect_intersections[rays_offset];

    if (direction_light_intersection.is_intersect == 0) {
        color = compute_color_for_global_lights(
            inter,
            direction_light_intersection,
            model,
            primary_ray
        );
    }
    else {
        color = vec3(0.0);
    }

    for (int i = 1; i - 1 < point_lights_count && i < MAX_INDIRECT_RAYS; i++) {
        Intersection shadow_intersection = indirect_intersections[rays_offset + i];
        PointLight light = point_lights[i - 1];
        vec3 direction = light.position - inter.point;

        if (shadow_intersection.is_intersect == 1) {
            continue;
        }
        color = color + compute_color_for_point_light(
            shadow_intersection.normal,
            normalize(direction),
            light,
            model.albedo,
            length(direction)
        );
    }

    return color;
}

void lights(uint rays_offset, Intersection inter, Ray primary_ray, ivec2 pos) {
    ModelInfo model = models[inter.model_id];
    if (model.material == MATERIAL_DIFFUSE) {
        vec3 color = compute_color_diffuse_material(rays_offset, model, inter, primary_ray);

        imageStore(resultImage, pos, vec4(color, 0.0));
    }
}

void main() {
    uint idx = gl_GlobalInvocationID.x;

    Intersection inter = primary_rays_intersections[idx];
    Ray primary_ray = primary_rays[idx];

    ivec2 pos = ivec2(idx % screen.x, idx / screen.x);

    if (inter.is_intersect == 1) {
        lights(idx * MAX_INDIRECT_RAYS, inter, primary_ray, pos);
    }
    else if (inter.is_intersect == 0) {
        imageStore(resultImage, pos, vec4(0.0, 0.7, 0.4, 0.0));
    }
    else {
        // unreachable
    }
}
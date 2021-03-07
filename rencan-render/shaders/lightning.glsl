#version 450

#extension GL_GOOGLE_include_directive : require

#include "include/defs.glsl"

layout(local_size_x = 64, local_size_y = 1, local_size_z = 1) in;

layout(set = 0, binding = 0) readonly uniform Info {
    uvec2 screen;
};
layout(std140, set = 0, binding = 1) readonly buffer PrimaryRays {
    Ray primary_rays[];
};
layout(set = 0, binding = 2, rgba8) writeonly uniform image2D resultImage;
layout(std140, set = 0, binding = 3) readonly buffer PrimaryIntersections {
    Intersection primary_rays_intersections[];
};
layout(std140, set = 0, binding = 4) readonly uniform DirectLightInfo {
    DirectLight global_light;
};
layout(std140, set = 0, binding = 5) readonly uniform PointLightsInfo {
    uint point_lights_count;
};
layout(std140, set = 0, binding = 6) readonly buffer PointLights {
    PointLight[] point_lights;
};
layout(std140, set = 1, binding = 0) readonly uniform SceneInfo {
    uint model_counts;
};
layout(std140, set = 1, binding = 1) readonly buffer ModelInfos {
    ModelInfo[] models;
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

    vec3 color = albedo / PI * global_light.intensity * global_light.color * max(dot(normal, -global_light.direction.xyz), 0.0);

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

Ray make_shadow_ray_for_direction_light(Intersection inter, Ray previous) {
    vec3 point = inter.point + inter.normal * 0.001;

    return Ray(point, vec4(-global_light.direction.xyz, 0.0), 1.0 / 0.0);
}

Ray make_shadow_ray_for_point_light(Intersection inter, Ray previous, PointLight light) {
    vec3 direction_ray = light.position - inter.point;

    vec3 point = inter.point + inter.normal * 0.001;

    return Ray(point, vec4(direction_ray, 0.0), length(direction_ray));
}

vec3 compute_color_diffuse_material(ModelInfo model, Intersection inter, Ray primary_ray) {
    vec3 normal = inter.normal;
    Intersection global_light_inter = trace(
        make_shadow_ray_for_direction_light(inter, primary_ray)
    );

    vec3 color;

    if (global_light_inter.is_intersect == 0) {
        color = compute_color_for_global_lights(
            normal,
            inter,
            global_light_inter,
            model,
            primary_ray
        );
    }
    else {
        color = vec3(0.0);
    }

    for (int i = 0; i < point_lights_count; i++) {
        PointLight light = point_lights[i];
        vec3 light_dir = light.position - inter.point;
        float distance_to_light = length(light_dir);

        Intersection shadow_intersection = trace(
            make_shadow_ray_for_point_light(
                inter,
                primary_ray,
                light
            )
        );
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

    return color;
}

void lights(uint idx, Intersection inter, Ray primary_ray, ivec2 pos) {
    ModelInfo model = models[inter.model_id];
    uvec3 index = indexes[inter.triangle_idx];

    vec3 color = compute_color_diffuse_material(model, inter, primary_ray);

    imageStore(resultImage, pos, vec4(color, 0.0));
}

void main() {
    uint idx = gl_GlobalInvocationID.x;

    Intersection inter = primary_rays_intersections[idx];
    Ray primary_ray = primary_rays[idx];

    ivec2 pos = ivec2(idx % screen.x, idx / screen.x);

    if (inter.is_intersect == 1) {
        lights(idx, inter, primary_ray, pos);
    }
    else if (inter.is_intersect == 0) {
        imageStore(resultImage, pos, vec4(0.0, 0.7, 0.4, 0.0));
    }
    else {
        // unreachable
    }
}
#version 450

#extension GL_GOOGLE_include_directive : require

#include "include/defs.glsl"

const uint SPECULAR_EXPONENT = 200;

layout(constant_id = 1) const uint SAMPLING = 0;
layout(constant_id = 2) const uint MAX_BOUNCES = 16;
const uint SAMPLING_MULT = 2;

layout(local_size_x_id = 0, local_size_y = 1, local_size_z = 1) in;

layout(set = 0, binding = 0) readonly uniform Info {
    uvec2 screen;
};
layout(std140, set = 0, binding = 1) readonly uniform Camera {
    vec3 pos;
    mat3 rotation;
    float fov;
};

layout(std140, set = 1, binding = 0) readonly buffer Intersections {
    Intersection intersections[];
};

layout(std140, set = 2, binding = 0) readonly uniform SceneInfo {
    uint model_counts;
};
layout(std140, set = 2, binding = 1) readonly buffer ModelInfos {
    ModelInfo[] models;
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

layout(std140, set = 3, binding = 0) readonly buffer SphereModelsInfo {
    uint sphere_models_count;
};
layout(std140, set = 3, binding = 1) readonly buffer SphereModels {
    ModelInfo sphere_models[];
};
layout(std140, set = 3, binding = 2) readonly buffer Spheres {
    Sphere[] spheres;
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

    vec3 light_dir = -global_light.direction;

    vec3 ray_direction = (inverse(isometry) * vec4(primary_ray.direction, 0.0)).xyz;

    vec3 color = global_light.intensity * global_light.color * max(dot(normal, -global_light.direction), 0.0);

    return color;
}

vec3 compute_color_for_point_light(
    vec3 normal,
    vec3 light_dir,
    PointLight light,
    float distance
) {
    vec3 intensity = light.intensity * light.color / (4 * PI * distance * distance);

    vec3 color = intensity * max(dot(normal, -light_dir), 0.0);
    return intensity;
}

Ray make_shadow_ray_for_direction_light(Intersection inter, Ray previous) {
    return Ray(inter.point, -global_light.direction, 1.0 / 0.0);
}

Ray make_shadow_ray_for_point_light(Intersection inter, Ray previous, PointLight light) {
    vec3 direction_ray = light.position - inter.point;

    return Ray(inter.point, normalize(direction_ray), length(direction_ray));
}

float compute_specular_component(vec3 primary_ray, vec3 light_ray, vec3 surface_normal) {
    vec3 reflected_ray = reflect(light_ray, surface_normal);
    float specular_component = pow(max(dot(reflected_ray, primary_ray), 0.0), SPECULAR_EXPONENT);
    return specular_component;
}

vec3 compute_specular_color(vec3 primary_ray, vec3 light_ray, vec3 surface_normal, vec3 light_color) {
    float component = compute_specular_component(primary_ray, light_ray, surface_normal);
    return light_color * component;
}

vec3 compute_color_diffuse_material(ModelInfo model, Intersection inter, Ray primary_ray) {
    vec3 normal = inter.normal;
    Ray global_light_ray = make_shadow_ray_for_direction_light(inter, primary_ray);
    Intersection global_light_inter = trace_first(
        global_light_ray,
        0
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
        /*color += model.specular * compute_specular_color(
            primary_ray.direction,
            global_light_ray.direction,
            inter.normal,
            global_light.color * global_light.intensity
        );*/
    }
    else {
        color = vec3(0.0);
    }

    for (int i = 0; i < point_lights_count; i++) {
        PointLight light = point_lights[i];
        vec3 light_dir = light.position - inter.point;
        float distance_to_light = length(light_dir);

        Ray light_ray = make_shadow_ray_for_point_light(
            inter,
            primary_ray,
            light
        );
        Intersection shadow_intersection = trace_first(
            light_ray,
            0
        );
        if (shadow_intersection.is_intersect == 1) {
            continue;
        }
        color += compute_color_for_point_light(
            normal,
            light_dir,
            light,
            distance_to_light
        );
        /*color += model.specular * compute_specular_color(
            primary_ray.direction,
            light_ray.direction,
            inter.normal,
            light.color * light.intensity
        );*/
    }

    return color;
}

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

vec3 lights_without_bounces(uint idx, Intersection inter, Ray primary_ray) {
    ModelInfo model = inter.model;

    bool computed = false;
    vec3 color = vec3(0.0);
    for (int i = 0; !computed && i < 100; i++) {
        switch (model.material) {
            case MATERIAL_DIFFUSE:
                color = model.albedo / PI * compute_color_diffuse_material(model, inter, primary_ray);
                computed = true;
                break;
            case MATERIAL_MIRROR:
                vec3 next_direction = reflect(primary_ray.direction, inter.normal);
                Ray reflect_ray = Ray(inter.point, next_direction, 1.0 / 0.0);
                Intersection mirror_inter = trace(reflect_ray, 0);
                if (mirror_inter.is_intersect == 0.0) {
                    color = vec3(0.0, 0.3, 0.8);
                    computed = true;
                }
                else {
                    inter = mirror_inter;
                    primary_ray = reflect_ray;
                    model = mirror_inter.model;
                }
                break;
            default:
                color = vec3(0.0, 0.0, 1.0);
                break;
        }
    }

    return color;
}

vec3 lights(uint idx, Intersection inter, Ray primary_ray) {
    ModelInfo model = inter.model;

    bool computed = false;
    vec3 color = vec3(0.0);
    for (int i = 0; !computed && i < 100; i++) {
        switch (model.material) {
            case MATERIAL_DIFFUSE:
                vec3 direct_lightning = compute_color_diffuse_material(model, inter, primary_ray);
                computed = true;

                vec3 indirect_color = vec3(0.0);

                float r1;
                float r2 = inter.distance * length(inter.point);

                for (int s = 0; s < MAX_BOUNCES; s++) {
                    r1 = rand(vec2(r2, s * 17));
                    r2 = rand(vec2(r1, inter.distance));
                    vec3 next_ray_direction = uniform_sample_hemisphere(r1, r2);
                    mat3 transf = create_coordinate_system(inter.normal);

                    vec3 next_ray_direction_global = transf * next_ray_direction;

                    Ray next_ray = Ray(inter.point, next_ray_direction_global, 1.0 / 0.0);

                    Intersection next_inter = trace(next_ray, 0);
                    if (next_inter.is_intersect == 1.0) {
                        indirect_color += r1 * compute_color_diffuse_material(next_inter.model, next_inter, next_ray) / (1 / (PI * 2));
                    }
                }
                indirect_color /= MAX_BOUNCES;
                color = (direct_lightning / PI + 2 * indirect_color) * model.albedo;

                break;
            case MATERIAL_MIRROR:
                vec3 next_direction = reflect(primary_ray.direction, inter.normal);
                Ray reflect_ray = Ray(inter.point, next_direction, 1.0 / 0.0);
                Intersection mirror_inter = trace(reflect_ray, 0);
                if (mirror_inter.is_intersect == 0.0) {
                    color = vec3(0.0, 0.3, 0.8);
                    computed = true;
                }
                else {
                    inter = mirror_inter;
                    primary_ray = reflect_ray;
                    model = mirror_inter.model;
                }
                break;
            default:
                color = vec3(0.0, 0.0, 1.0);
                break;
        }
    }

    return color;
}

Ray compute_primary_ray(
    uvec2 screen,
    uvec2 this_point,
    float fov,
    vec3 camera_origin,
    mat3 camera_rotation
) {
    float scale = tan(fov / 2);
    float aspect_ratio = float(screen.x) / float(screen.y);

    vec3 origin = camera_origin;

    float x = (2 * ((this_point.x + 0.5) / float(screen.x)) - 1) * aspect_ratio * scale;
    float y = (1 - 2 * ((this_point.y + 0.5) / float(screen.y))) * scale;

    vec3 direction = normalize(camera_rotation * vec3(x, y, -1.0));

    return Ray(origin, direction, 1.0 / 0.0);
}

vec3 compute_color_for_pixel(uint idx, uvec2 screen, uvec2 pixel_pos) {
    Ray primary_ray = compute_primary_ray(
        screen,
        pixel_pos,
        fov,
        pos,
        rotation
    );
    Intersection inter = trace(primary_ray, 0);

    vec3 color;
    if (inter.is_intersect == 1) {
        color = lights(idx, inter, primary_ray);
    }
    else if (inter.is_intersect == 0) {
        color = vec3(0.0, 0.3, 0.8);
    }
    else {
        // unreachable
    }
    return color;
}

vec3 tracing_with_sampling() {
    uint idx = gl_GlobalInvocationID.x * SAMPLING_MULT;

    uvec2 local_screen = screen * SAMPLING_MULT;

    vec3 color = vec3(0.0);

    for (int i=0; i<SAMPLING_MULT * SAMPLING_MULT; i++) {
        uvec2 local_pixel_pos = uvec2(
            idx % local_screen.x + i % SAMPLING_MULT,
            (idx * SAMPLING_MULT) / local_screen.x + i / SAMPLING_MULT
        );
        color += compute_color_for_pixel(idx, local_screen, local_pixel_pos);
    }
    color /= SAMPLING_MULT * SAMPLING_MULT;

    return color;
}

void main() {
    uint idx = gl_GlobalInvocationID.x;
    vec3 color;

    if (SAMPLING == 1) {
        ivec2 pixel_pos = ivec2(idx % screen.x, idx / screen.x);

        color = tracing_with_sampling();
        imageStore(resultImage, pixel_pos, vec4(color, 1.0));
    }
    else {
        Intersection inter = intersections[idx];
        if (inter.is_intersect != 1) {
            return;
        }
        ivec2 pixel_pos = ivec2(inter.pixel_id % screen.x, inter.pixel_id / screen.x);
        Ray primary_ray = inter.ray;
        color = lights(idx, inter, primary_ray);
        imageStore(resultImage, pixel_pos, vec4(color, 1.0));
    }
}
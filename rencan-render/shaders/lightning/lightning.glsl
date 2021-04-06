#version 450

#extension GL_GOOGLE_include_directive : require

#include "../include/defs.glsl"

const uint SPECULAR_EXPONENT = 200;

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
    LightIntersection intersections[];
};

layout(set = 2, binding = 0, r32ui) uniform uimage2D resultImage;

#define PI radians(180)

vec3 compute_diffuse_component(
    LightIntersection inter
) {
    vec3 normal = inter.inter.normal;
    mat4 isometry = inter.inter.model.isometry;
    float albedo = inter.inter.model.albedo;

    vec3 light_dir = inter.inter.ray.direction;

    vec3 color = albedo / PI * inter.light_intensity * max(dot(normal, light_dir), 0.0);

    return color;
}

vec3 compute_diffuse_color(
    LightIntersection inter
) {
    float diffuse_coef = inter.inter.model.diffuse;

    return diffuse_coef * compute_diffuse_component(inter);
}
/* TODO: specular component
requires primary ray direction
float compute_specular_component(vec3 primary_ray, vec3 light_ray, vec3 surface_normal) {
    vec3 reflected_ray = reflect(light_ray, surface_normal);
    float specular_component = pow(max(dot(reflected_ray, primary_ray), 0.0), SPECULAR_EXPONENT);
    return specular_component;
}

vec3 compute_specular_color(vec3 primary_ray, vec3 light_ray, vec3 surface_normal, vec3 light_color) {
    float component = compute_specular_component(primary_ray, light_ray, surface_normal);
    return light_color * component;
}
*/
vec3 compute_color_diffuse_material(LightIntersection inter) {
    return compute_diffuse_color(inter);
}

vec3 lights(LightIntersection inter) {
    vec3 color = compute_color_diffuse_material(inter);

    return color;
}

void main() {
    uint idx = gl_GlobalInvocationID.x;

    LightIntersection inter = intersections[idx];
    if (inter.inter.is_intersect != 1) {
        return;
    }
    ivec2 pixel_pos = ivec2(inter.inter.pixel_id % screen.x, inter.inter.pixel_id / screen.x);

    vec3 normal = inter.inter.normal;
    mat4 isometry = inter.inter.model.isometry;
    float albedo = inter.inter.model.albedo;

    vec3 light_dir = inter.light_ray.direction;

    vec3 color = albedo / PI * inter.light_intensity * max(dot(normal, light_dir), 0.0);
    //color = albedo / PI * inter.light_intensity;

    // argb
    imageAtomicAdd(resultImage, pixel_pos, vec4_color_to_uint(vec4(0.0, color)));
}

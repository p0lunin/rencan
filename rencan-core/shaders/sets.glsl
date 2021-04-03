#version 450

#extension GL_GOOGLE_include_directive : require

#include "../../rencan-render/shaders/include/defs.glsl"

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

layout(set = 5, binding = 0, rgba8) readonly uniform image2D resultImage;

layout(set = 6, binding = 0) readonly buffer Workgroups {
    uint x_dimension;
    uint y_dimension;
    uint z_dimension;
};

void main() {
    uvec2 s = screen;
    vec3 v = pos;
    Intersection i = intersections[0];
    uint sdg = model_counts;
    ModelInfo sd = models[0];
    vec3 vs = vertices[0];
    uvec3 fsd = indexes[0];
    HitBoxRectangle hr = hit_boxes[0];
    uint sdf = sphere_models_count;
    ModelInfo hg = sphere_models[0];
    Sphere sfhgt = spheres[0];
    DirectLight gl = global_light;
    uint ps = point_lights_count;
    PointLight pghf = point_lights[0];
    vec4 fds = imageLoad(resultImage, ivec2(0.0));
    uint fghfj = x_dimension;
}
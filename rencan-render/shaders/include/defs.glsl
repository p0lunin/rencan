struct Ray {
    vec3 origin;
    vec3 direction;
    float max_distance;
};

struct ModelMaterial {
    uint material;
    float albedo;
    float diffuse;
    float specular;
};

struct ModelInfo {
    mat4 isometry;
    mat4 inverse_isometry;
    uint model_id;
    uint vertices_length;
    uint indexes_length;
    ModelMaterial material;
};

struct Intersection {
    vec3 point;
    vec3 normal;
    ModelMaterial model_material;
    float distance;
    Ray ray;
    uint pixel_id;
};

struct LightRay {
    Ray ray;
    vec3 light_intensity;
    uint inter_id;
};

Intersection intersection_succ(
    vec3 point,
    vec3 normal,
    ModelMaterial model_material,
    float distance,
    Ray ray,
    uint pixel_id
) {
    return Intersection(point, normal, model_material, distance, ray, pixel_id);
}

struct DirectLight {
    vec3 color;
    vec3 direction;
    float intensity;
};

struct Sphere {
    vec3 center;
    float radius;
};

struct HitBoxRectangle {
    vec3 min;
    vec3 max;
};

struct PointLight {
    vec3 color;
    vec3 position;
    float intensity;
};

const uint MATERIAL_DIFFUSE = 1;
const uint MATERIAL_MIRROR = 2;
const uint MATERIAL_REFRACT = 3;

#define PI radians(180)

float rand(vec2 co){
    return fract(sin(dot(co.xy ,vec2(12.9898,78.233))) * 43758.5453);
}

vec3 compute_light_color(
    ModelMaterial model,
    vec3 light_intensity,
    vec3 inter_normal,
    vec3 light_direction
) {
    vec3 color = model.diffuse * model.albedo / PI * light_intensity * max(dot(inter_normal, light_direction), 0.0);
    return color;
}

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

float compute_specular_component(vec3 primary_ray, vec3 light_ray, vec3 surface_normal, float specular_exponent) {
    vec3 reflected_ray = reflect(light_ray, surface_normal);
    float specular_component = pow(max(dot(reflected_ray, primary_ray), 0.0), specular_exponent);
    return specular_component;
}

vec3 compute_specular_color(vec3 primary_ray, vec3 light_ray, vec3 surface_normal, vec3 light_color, float specular_exponent) {
    float component = compute_specular_component(primary_ray, light_ray, surface_normal, specular_exponent);
    return light_color * component;
}

vec3 compute_light_color(
    ModelMaterial model,
    vec3 light_intensity,
    vec3 inter_normal,
    vec3 light_direction,
    vec3 eye_ray_direction
) {
    vec3 color = model.diffuse * model.albedo / PI * light_intensity * max(dot(inter_normal, light_direction), 0.0);
    color = model.specular *
        compute_specular_color(eye_ray_direction, light_direction, inter_normal, light_intensity, 200) + color;
    return color;
}

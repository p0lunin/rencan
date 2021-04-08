struct Ray {
    vec3 origin;
    vec3 direction;
    float max_distance;
};

struct ModelInfo {
    mat4 isometry;
    uint model_id;
    uint vertices_length;
    uint indexes_length;
    uint material;
    float albedo;
    float diffuse;
    float specular;
};

struct Intersection {
    vec3 point;
    vec3 normal;
    vec2 barycentric_coords;
    uint is_intersect;
    ModelInfo model;
    uint triangle_idx;
    uint vertices_offset;
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
    ModelInfo model,
    uint triangle_idx,
    uint vertices_offset,
    vec2 barycentric_coords,
    float distance,
    Ray ray,
    uint pixel_id
) {
    return Intersection(point, normal, barycentric_coords, 1, model, triangle_idx, vertices_offset, distance, ray, pixel_id);
}

Intersection intersection_none() {
    Intersection intersect;
    intersect.is_intersect = 0;
    return intersect;
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

uint vec4_color_to_uint(vec4 color) {
    uvec4 bytes = uvec4(clamp(color, 0, 1) * 255);
    uint integer_value = (bytes[0] << 24) | (bytes[1] << 16) | (bytes[2] << 8) | (bytes[3]);
    return integer_value;
}

vec4 uint_color_to_vec4(uint color) {
    uvec4 bytes = uvec4((color >> 24) & 255, (color >> 16) & 255, (color >> 8) & 255, color & 255);
    return vec4(bytes) / 255;
}

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
    ModelInfo model,
    vec3 light_intensity,
    vec3 inter_normal,
    vec3 light_direction,
    vec3 eye_ray_direction
) {
    vec3 color = model.albedo / PI * light_intensity * max(dot(inter_normal, light_direction), 0.0);
    color += model.specular *
        compute_specular_color(eye_ray_direction, light_direction, inter_normal, light_intensity, 200);
    return color;
}

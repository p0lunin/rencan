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

struct LightIntersection {
    Intersection inter;
    Ray light_ray;
    vec3 light_intensity;
};

struct LightRay {
    Intersection previous_intersection;
    Ray ray;
    vec3 light_intensity;
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

uint vec4_color_to_uint(vec4 color) {
    ivec4 bytes = ivec4(color * 255);
    uint integer_value = (bytes.r << 24) | (bytes.g << 16) | (bytes.b << 8) | (bytes.a);
    return integer_value;
}

float rand(vec2 co){
    return fract(sin(dot(co.xy ,vec2(12.9898,78.233))) * 43758.5453);
}

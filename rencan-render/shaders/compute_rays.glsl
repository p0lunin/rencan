#version 450

struct Ray {
    vec3 origin;
    vec4 direction;
};

struct Intersection {
    vec2 barycentric_coords;
    uint is_intersect;
    uint model_id;
    uint triangle_idx;
    float distance;
};

layout(local_size_x = 64, local_size_y = 1, local_size_z = 1) in;

layout(set = 0, binding = 0) uniform Info {
    uvec2 screen;
};
layout(std140, set = 0, binding = 1) uniform Camera {
    vec3 pos;
    mat3 rotation;
    float x_angle;
    float y_angle;
} camera;
layout(std140, set = 0, binding = 2) buffer RaysInfo {
    Ray data[];
} rays;
layout(set = 0, binding = 3, rgba8) uniform image2D resultImage;
layout(std140, set = 0, binding = 4) buffer Intersections {
    Intersection intersections[];
};

uint compute_x(uint screen_width) {
    return gl_GlobalInvocationID.x % screen_width;
}

uint compute_y(uint screen_width) {
    return gl_GlobalInvocationID.x / screen_width;
}

vec2 compute_origin(float x, float y) {
    y = float(screen.y) / 2.0;
    x = x * (screen.x - screen.y) / screen.x + screen.y / 2;
    return vec2(x, y);
}

vec3 from_raster_space_to_coords(vec2 coords, vec2 step, vec2 max_deviation) {
    float x = coords.x * step.x * 2 * max_deviation.x - max_deviation.x;
    float y = -(coords.y * step.y * 2 * max_deviation.y) + max_deviation.y;
    return vec3(x, y, 0.0);
}

void main() {
    uint screen_width = screen.x;
    uint screen_height = screen.y;
    float x_props = float(screen_width) / float(screen_height);
    if (x_props <= 1.0) {
        x_props = 1.0;
    }
    x_props = x_props + 0.25;

    uint idx = gl_GlobalInvocationID.x;

    vec2 this_point = vec2(compute_x(screen_width), compute_y(screen_width));

    vec2 step = vec2(1.0/(screen_width - 1), 1.0/(screen_height - 1));
    vec2 max_deviation = vec2(tan(camera.x_angle/2) * x_props, tan(camera.y_angle/2));

    vec2 origin_for_ray = compute_origin(this_point.x, this_point.y);
    vec3 ray_origin_local = from_raster_space_to_coords(origin_for_ray, step, max_deviation);

    vec3 deviation_local = from_raster_space_to_coords(this_point, step, max_deviation);
    deviation_local.z = -1.0;

    vec3 deviation_global = camera.rotation * deviation_local;

    vec4 ray_direction = vec4(
        deviation_global,
        0.0
    );

    vec3 origin = camera.pos + camera.rotation * ray_origin_local;

    rays.data[idx] = Ray(origin, ray_direction);
}
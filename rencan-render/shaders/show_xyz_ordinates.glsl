#version 450

layout(local_size_x = 1, local_size_y = 1, local_size_z = 1) in;

layout(set = 0, binding = 0) uniform Screen {
    uvec2 screen;
};
layout(set = 0, binding = 1) uniform Origin {
    vec3 origin;
};
layout(set = 0, binding = 2) buffer Rays {
    vec4 rays[];
};
layout(set = 0, binding = 3, rgba8) uniform image2D out_image;

struct IntersectResult {
    float distance;
    bool intersect;
};

IntersectResult not_intersect() {
    return IntersectResult(0.0, false);
}

IntersectResult intersect(float distance) {
    return IntersectResult(distance, true);
}

const float eps = 0.008;

bool eqf(float f1, float f2) {
    return abs(f1 - f2) < eps;
}

bool try_intersect_x(vec3 ray, float t) {
    return
        eqf(ray.y * t + origin.y, 0.0) &&
        eqf(ray.z * t + origin.z, 0.0) &&
        origin.x + ray.x * t >= 0;
}

IntersectResult check_x_y(vec3 ray) {
    float t = (-origin.y) / ray.y;
    if (t > 0 && try_intersect_x(ray, t)) {
        return intersect(t);
    }
    else {
        return not_intersect();
    }
}
IntersectResult check_x_z(vec3 ray) {
    float t = (-origin.z) / ray.z;
    if (t > 0 && try_intersect_x(ray, t)) {
        return intersect(t);
    }
    else {
        return not_intersect();
    }
}

IntersectResult intersect_x(vec3 ray) {
    IntersectResult res_out = intersect(3.402823466e+38);
    if (!(eqf(ray.y, 0.0) && eqf(origin.y, 0.0))) {
        IntersectResult res = check_x_y(ray);
        if (res.intersect) {
            res_out.distance = res.distance;
        }
        else {
            return not_intersect();
        }
    }
    if (!(eqf(ray.z, 0.0) && eqf(origin.z, 0.0))) {
        IntersectResult res = check_x_z(ray);
        if (res.intersect) {
            res_out.distance = min(res.distance, res_out.distance);
        }
        else {
            return not_intersect();
        }
    }
    return res_out;
}

bool try_intersect_y(vec3 ray, float t) {
    return
        eqf(ray.x * t + origin.x, 0.0) &&
        eqf(ray.z * t + origin.z, 0.0) &&
        origin.y + ray.y * t >= 0;
}

bool check_y_x(vec3 ray) {
    float t = (-origin.x) / ray.x;
    return t > 0 && try_intersect_y(ray, t);
}
bool check_y_z(vec3 ray) {
    float t = (-origin.z) / ray.z;
    return t > 0 && try_intersect_y(ray, t);
}

bool intersect_y(vec3 ray) {
    bool intersect = true;
    if (!(eqf(ray.x, 0.0) && eqf(origin.x, 0.0)))
        intersect = intersect && check_y_x(ray);
    if (!(eqf(ray.z, 0.0) && eqf(origin.z, 0.0)))
        intersect = intersect && check_y_z(ray);
    return intersect;
}

bool try_intersect_z(vec3 ray, float t) {
    return
        eqf(ray.x * t + origin.x, 0.0) &&
        eqf(ray.y * t + origin.y, 0.0) &&
        origin.z + ray.z * t >= 0;
}

bool check_z_x(vec3 ray) {
    float t = (-origin.x) / ray.x;
    return (t > 0 && try_intersect_z(ray, t));
}
bool check_z_y(vec3 ray) {
    float t = (-origin.y) / ray.y;
    return (t > 0 && try_intersect_z(ray, t));
}

bool intersect_z(vec3 ray) {
    bool intersect = true;
    if (!(eqf(ray.x, 0.0) && eqf(origin.x, 0.0)))
        intersect = intersect && check_z_x(ray);
    if (!(eqf(ray.y, 0.0) && eqf(origin.y, 0.0)))
        intersect = intersect && check_z_y(ray);
    return intersect;
}

void main() {
    uint idx = gl_GlobalInvocationID.x;
    vec3 ray = rays[idx].xyz;

    ivec2 pos = ivec2(idx % screen.x, idx / screen.x);

    vec4 pixel = imageLoad(out_image, pos);

    float distance = pixel.w;

    IntersectResult x_res = intersect_x(ray);
    if (x_res.intersect) {
        if (x_res.distance < distance) {
            distance = x_res.distance;
            imageStore(out_image, pos, vec4(1.0, 0.0, 0.0, distance));
        }
    }
    else if (intersect_y(ray)) {
        imageStore(out_image, pos, vec4(0.0, 1.0, 0.0, distance));
    }
    else if (intersect_z(ray)) {
        imageStore(out_image, pos, vec4(0.0, 0.0, 1.0, distance));
    }
}

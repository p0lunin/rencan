#version 450

layout(local_size_x = 64, local_size_y = 1, local_size_z = 1) in;

layout(set = 0, binding = 0) uniform Info {
    uvec2 screen;
};
layout(std140, set = 0, binding = 1) uniform Origin {
    vec3 origin;
    mat3 rotation;
};
layout(set = 0, binding = 2) buffer Rays {
    vec3 rays[];
};
layout(set = 0, binding = 3, rgba8) uniform image2D resultImage;
layout(set = 1, binding = 0) buffer Vertices {
    vec3[] vertices;
};
layout(std140, set = 1, binding = 1) buffer IndexesLength {
    uint indexes_length;
};
layout(std140, set = 1, binding = 2) buffer Indexes {
    uvec3[] indexes;
};

const float eps = 0.001;

bool eqf(float f1, float f2) {
    return abs(f1 - f2) < eps;
}

struct IntersectResult {
    vec2 barycentric_coords;
    float distance;
    bool intersect;
};

IntersectResult not_intersect() {
    return IntersectResult(vec2(0.0), 0.0, false);
}
IntersectResult ret_intersect(vec2 coords, float t) {
    return IntersectResult(coords, t, true);
}

IntersectResult intersect(vec3 ray, vec3[3] triangle) {
    vec3 v0v1 = triangle[1] - triangle[0];
    vec3 v0v2 = triangle[2] - triangle[0];
    vec3 pvec = cross(ray, v0v2);
    float det = dot(v0v1, cross(ray, v0v2));

    if (det < eps) return not_intersect();

    float inv_det = 1.0 / det;

    vec3 tvec = origin - triangle[0];
    float u = dot(tvec, pvec) * inv_det;
    if (u < 0 || u > 1) return not_intersect();

    vec3 vvec = cross(tvec, v0v1);
    float v = dot(ray, vvec) * inv_det;
    if (v < 0 || u + v > 1) return not_intersect();

    float t = dot(v0v2, vvec) * inv_det;

    return ret_intersect(vec2(u, v), t);
}

void main() {
    uint idx = gl_GlobalInvocationID.x;

    ivec2 pos = ivec2(idx % screen.x, idx / screen.x);

    float distance = imageLoad(resultImage, pos).w;
    if (distance == 0.0) {
        distance = 1.0 / 0.0;
    }
    bool need_rewrite = false;
    vec4 out_color = vec4(0.0, 0.0, 0.0, 0.0);

    for (int i = 0; i < indexes_length; i++) {
        uvec3 index = indexes[i];
        vec3[3] triangles = vec3[](vertices[index.x], vertices[index.y], vertices[index.z]);
        IntersectResult res = intersect(rays[idx].xyz, triangles);
        if (res.intersect && res.distance < distance) {
            need_rewrite = true;
            out_color = vec4(
                res.barycentric_coords.x,
                res.barycentric_coords.y,
                1 - res.barycentric_coords.x - res.barycentric_coords.y,
                res.distance
            );
        }
    }

    if (need_rewrite) {
        imageStore(resultImage, pos, out_color);
    }
}
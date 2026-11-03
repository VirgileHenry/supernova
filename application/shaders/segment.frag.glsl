#version 460

struct CameraUniform {
    mat4 view;
    mat4 proj;
};

struct SegmentInstance {
    mat4 transform;
    mat4 transform_inv;
    vec4 aabb_min;
    vec4 aabb_max;
    uint shell_offset;
    uint shell_length;
    uint interior_offset;
    uint interior_length;
};

struct CsgNode {
    uint opcode;
    uint child_count;
    uint _pad0;
    uint _pad1;
    float data[12];
};

// Camera uniform buffer
layout(set = 0, binding = 0, std140) uniform CameraBuffer {
    CameraUniform camera;
};

// Instance storage buffer
layout(set = 0, binding = 1, std430) readonly buffer InstanceBuffer {
    SegmentInstance instances[];
};

// Segments storage buffer
layout(set = 0, binding = 2, std430) readonly buffer CsgBuffer {
    CsgNode csg_nodes[];
};

// Get the instance index from the frgament shader
layout(location = 0) flat in uint in_instance_index;
// Get the fragment ndc the frgament shader
layout(location = 1) in vec2 in_ndc;

// Output color
layout(location = 0) out vec4 outColor;

struct Ray {
    vec3 origin;
    vec3 direction;
};

Ray get_camera_ray(SegmentInstance instance);
bool ray_aabb_intersection(Ray ray, vec3 aabb_min, vec3 aabb_max, out float t_enter, out float t_exit);

float sdf(vec3 at, uint start_node, uint csg_node_count);
float segment_sdf(SegmentInstance instance, vec3 at);
float cylinder_sdf(vec3 at, CsgNode node);

void main()
{
    SegmentInstance instance = instances[in_instance_index];
    Ray ray = get_camera_ray(instance);

    // init to black ??
    outColor = vec4(0.1, 0.1, 0.1, 1.0);

    float t_aabb_enter;
    float t_aabb_exit;
    bool aabb_hit = ray_aabb_intersection(ray, instance.aabb_min.xyz, instance.aabb_max.xyz, t_aabb_enter, t_aabb_exit);

    if (aabb_hit) {
        outColor = vec4(0.4, 0.1, 0.4, 1.0);

        // the hard number over the max numbers of iterations.
        // the more the better quality (avoid ome artifacts when we struggle to hit the csg)
        // but also the more expensive it gets.
        uint max_iter = 200;
        // how close to the surface we need to be in order to hit.
        // the less the better quality, but the more expensive.
        float hit_eps = 0.001;

        float t = max(t_aabb_enter, 0.0);

        for (uint i = 0; i < max_iter; i++) {
            vec3 eval_point = ray.origin + ray.direction * t;
            float scene_sdf = segment_sdf(instance, eval_point);

            if (scene_sdf < hit_eps) {
                outColor = vec4(0.1, 1.0, 0.1, 1.0);
                break;
            }

            t += scene_sdf;
            if (t > t_aabb_exit) {
                // Out of the aabb, stop marching
                discard;
                break;
            }
        }
    } else {
        discard;
    }
}

Ray get_camera_ray(SegmentInstance instance) {
    mat4 inv_view_proj = inverse(camera.proj * camera.view);

    // Vulkan NDC depth range: [0, 1].
    vec4 near_clip = vec4(in_ndc, 0.0, 1.0);
    vec4 far_clip = vec4(in_ndc, 1.0, 1.0);

    // Unproject into world space.
    vec4 near_world = inv_view_proj * near_clip;
    vec4 far_world = inv_view_proj * far_clip;

    near_world /= near_world.w;
    far_world /= far_world.w;

    // Transform into segment-local space.
    vec3 near_local = (instance.transform_inv * near_world).xyz;
    vec3 far_local = (instance.transform_inv * far_world).xyz;

    Ray ray;
    ray.origin = near_local;
    ray.direction = normalize(far_local - near_local);

    return ray;
}

bool ray_aabb_intersection(Ray ray, vec3 aabb_min, vec3 aabb_max, out float t_enter, out float t_exit) {
    vec3 inv_dir = 1.0 / ray.direction;

    vec3 t0 = (aabb_min - ray.origin) * inv_dir;
    vec3 t1 = (aabb_max - ray.origin) * inv_dir;

    vec3 t_min = min(t0, t1);
    vec3 t_max = max(t0, t1);

    t_enter = max(max(t_min.x, t_min.y), t_min.z);
    t_exit = min(min(t_max.x, t_max.y), t_max.z);

    // No intersection, or the box is entirely behind the ray.
    return t_exit >= max(t_enter, 0.0);
}

float segment_sdf(SegmentInstance instance, vec3 at) {
    float shell_sdf = sdf(at, instance.shell_offset, instance.shell_length);
    float interior_sdf = sdf(at, instance.interior_offset, instance.interior_length);

    return max(shell_sdf, -interior_sdf);
}

float sdf(vec3 at, uint start_node, uint csg_node_count) {
    uint stack_ptr = 0;
    float stack[16];

    for (uint i = 0; i < csg_node_count; i++) {
        CsgNode csg_node = csg_nodes[start_node + i];
        switch (csg_node.opcode) {
            // CYLINDER
            case 9:
            stack[stack_ptr] = cylinder_sdf(at, csg_node);
            stack_ptr += 1;
            break;
            default:
            return 0.;
        }
    }

    // The final result is at the top of the stack
    // We should have the stack pointer at 1 (one element) if the csg was well formed
    return stack[stack_ptr - 1];
}

float cylinder_sdf(vec3 at, CsgNode node) {
    vec3 a = vec3(node.data[0], node.data[1], node.data[2]);
    vec3 b = vec3(node.data[3], node.data[4], node.data[5]);
    float radius = node.data[6];

    vec3 ba = b - a;
    float baba = dot(ba, ba);

    // Point relative to cylinder centerline
    vec3 pa = at - a;

    // Distance along the cylinder axis
    float h = dot(pa, ba) / baba;

    // Distance from the cylinder axis
    float radial = length(pa - ba * h) - radius;

    // Distance to the capped cylinder's axial extent
    float axial = (abs(h - 0.5) - 0.5) * length(ba);

    // Combine radial and axial distances
    vec2 d = vec2(radial, axial);

    return length(max(d, 0.0)) + min(max(d.x, d.y), 0.0);
}

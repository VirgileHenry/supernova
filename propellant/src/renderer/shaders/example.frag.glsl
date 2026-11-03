#version 460

layout(location = 0) out vec4 outColor;

struct Ray {
    vec3 origin;
    vec3 direction;
};

/// noramalized_frag_pos should be between -1 and 1
Ray get_ray(vec2 noramalized_frag_pos) {
    vec3 cam_right = vec3(1.0, 0.0, 0.0);
    vec3 cam_up = vec3(0.0, -1.0, 0.0); // neg because vulkan has y going downward?
    vec3 cam_forward = vec3(0.0, 0.0, 1.0);

    float fovy = 3.141592 / 2;
    float cam_aspect_ratio = 476.0 / 535.0;

    // TODO: take cam fovy into account
    float tan_cam_fovy_halfed = tan(fovy * 0.5);
    float tan_cam_fovx_halfed = cam_aspect_ratio * tan_cam_fovy_halfed;

    float x = noramalized_frag_pos.x * tan_cam_fovx_halfed;
    float y = noramalized_frag_pos.y * tan_cam_fovy_halfed;

    // TODO: transform with cam position
    vec3 ray_direction = normalize(cam_forward + cam_right * x + cam_up * y);

    // ray position should be transformed along with the object position
    // TODO: transform with object position
    vec3 ray_position = vec3(0.0, 0.0, -4.0);

    return Ray(ray_position, ray_direction);
}

float scene_sdf(vec3 at) {
    // temp for now: sphere at center
    return length(at) - 1.0;
}

vec3 scene_normal(vec3 at) {
    // small enough for graphic precision, yet big enough to avoid noise artifacts
    float h = 0.00001;
    vec2 k = vec2(1.0, -1.0);
    vec3 normal = normalize(
            k.xyy * scene_sdf(at + k.xyy * h) +
                k.yyx * scene_sdf(at + k.yyx * h) +
                k.yxy * scene_sdf(at + k.yxy * h) +
                k.xxx * scene_sdf(at + k.xxx * h)
        );

    // TODO: switch normal from cam view space back to world space

    return normal;
}

void main() {
    outColor = vec4(0.1, 0.1, 0.1, 1.0);

    vec2 normalized_frag_pos = vec2(
            gl_FragCoord.x / 476.0 - 1.0, // hard coded my screen res for now
            gl_FragCoord.y / 535.0 - 1.0
        );

    Ray ray = get_ray(normalized_frag_pos);

    uint max_iter = 200;
    float hit_eps = 0.0001;

    // how close to the surface we need to be in order to hit.
    // the less the better quality, but the more expensive.
    vec3 eval_point = ray.origin;
    for (uint i = 0; i < max_iter; i++) {
        float scene_sdf = scene_sdf(eval_point);
        if (scene_sdf < hit_eps) {
            // it's a hit !
            vec3 normal = scene_normal(eval_point);
            outColor = vec4(normal, 1.0);
            break;
        }
        eval_point += ray.direction * scene_sdf;
    }
}

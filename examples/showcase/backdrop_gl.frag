uniform sampler2D u_texture;
uniform vec4 u_region;
uniform vec4 u_corners;
uniform vec4 u_capture;
uniform vec4 u_texels;
uniform vec3 u_effect; // lens displacement, bevel depth, polish
uniform float u_strength;
out vec4 pixel;

float surface_distance(vec2 local) {
    vec2 pair = local.y >= 0.0 ? u_corners.xy : u_corners.zw;
    float radius = min(local.x < 0.0 ? pair.x : pair.y, min(u_region.z, u_region.w));
    vec2 edge = abs(local) - u_region.zw + radius;
    return length(max(edge, vec2(0.0))) + min(max(edge.x, edge.y), 0.0) - radius;
}

void main() {
    vec2 point = gl_FragCoord.xy;
    vec2 local = point - u_region.xy;
    float distance_to_edge = surface_distance(local);
    float mask = 1.0 - smoothstep(-1.0, 0.0, distance_to_edge);
    if (mask <= 0.0) { discard; }

    // Frost samples the evenly blurred backdrop. Prism uses the normal of
    // the same rounded silhouette for a continuous, single-sample lens.
    float light = 0.0;
    if (u_effect.x > 0.0) {
        float depth = min(u_effect.y, min(u_region.z, u_region.w) * 0.82);
        float bevel = 1.0 - smoothstep(0.0, depth, -distance_to_edge);
        if (bevel > 0.0) {
            vec2 dx = vec2(1.0, 0.0);
            vec2 dy = vec2(0.0, 1.0);
            vec2 gradient = vec2(
                surface_distance(local + dx) - surface_distance(local - dx),
                surface_distance(local + dy) - surface_distance(local - dy)
            );
            vec2 normal = gradient / max(length(gradient), 0.001);
            point += normal * (u_effect.x * bevel * bevel);
            float facing = dot(normal, vec2(-0.6, 0.8));
            float edge = 1.0 - smoothstep(0.3, 3.0, -distance_to_edge);
            light = edge * u_effect.z * (0.27 * max(facing, 0.0) - 0.11 * max(-facing, 0.0));
        }
    }

    vec2 texel = (point - u_capture.xy) * u_capture.zw;
    vec2 uv = clamp(texel, vec2(0.5), u_texels.xy - 0.5) / u_texels.zw;
    vec3 color = clamp(texture(u_texture, uv).rgb + vec3(light), 0.0, 1.0);
    float alpha = mask * u_strength;
    pixel = vec4(color * alpha, alpha);
}

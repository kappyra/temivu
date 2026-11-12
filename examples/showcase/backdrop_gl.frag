uniform sampler2D u_texture;
uniform vec4 u_region;
uniform vec4 u_corners;
uniform vec4 u_capture;
uniform vec4 u_texels;
uniform vec3 u_effect; // lens displacement, bevel depth, polish
uniform float u_strength;
out vec4 pixel;

float surface_distance(vec2 local, out vec2 edge, out float arc_distance) {
    vec2 pair = local.y >= 0.0 ? u_corners.xy : u_corners.zw;
    float radius = min(local.x < 0.0 ? pair.x : pair.y, min(u_region.z, u_region.w));
    edge = abs(local) - u_region.zw + radius;
    arc_distance = length(max(edge, vec2(0.0)));
    return arc_distance + min(max(edge.x, edge.y), 0.0) - radius;
}

void main() {
    vec2 point = gl_FragCoord.xy;
    vec2 local = point - u_region.xy;
    vec2 edge;
    float arc_distance;
    float distance_to_edge = surface_distance(local, edge, arc_distance);
    float mask = 1.0 - smoothstep(-1.0, 0.0, distance_to_edge);
    if (mask <= 0.0) { discard; }

    // Frost samples the evenly blurred backdrop. Prism uses the normal of
    // the same rounded silhouette for a continuous, single-sample lens.
    float shade = 0.0;
    float specular = 0.0;
    if (u_effect.x > 0.0 || u_effect.z > 0.0) {
        float half_side = min(u_region.z, u_region.w);
        float depth = min(u_effect.y, half_side * 0.82);
        float inset = -distance_to_edge;
        float bevel = 1.0 - smoothstep(0.0, depth, inset);
        if (bevel > 0.0) {
            // Reproject in screen space. When the surface moves, each part
            // of its bevel bends a different portion of the live scene.
            vec2 side_normal = edge.x > edge.y ? vec2(1.0, 0.0) : vec2(0.0, 1.0);
            vec2 normal = arc_distance > 0.001 ? max(edge, vec2(0.0)) / arc_distance : side_normal;
            normal *= mix(vec2(-1.0), vec2(1.0), step(vec2(0.0), local));
            float displacement = min(u_effect.x, half_side * 0.72);
            point += normal * displacement * bevel * (0.65 + 0.35 * bevel);
            float facing = dot(normal, vec2(-0.6, 0.8));
            float rim = 1.0 - smoothstep(0.0, depth * 0.26, inset);
            float key = max(facing, 0.0);
            shade = rim * u_effect.z * (0.12 + 0.17 * max(-facing, 0.0));
            specular = rim * u_effect.z * (0.37 * key * key + 0.08 * bevel);
        }
    }

    vec2 texel = (point - u_capture.xy) * u_capture.zw;
    vec2 uv = clamp(texel, vec2(0.5), u_texels.xy - 0.5) / u_texels.zw;
    vec3 scene = texture(u_texture, uv).rgb;
    vec3 color = mix(scene * (1.0 - shade), vec3(1.0), specular);
    float alpha = mask * u_strength;
    pixel = vec4(color * alpha, alpha);
}

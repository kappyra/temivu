uniform sampler2D u_texture;
uniform sampler2D u_detail;
uniform vec4 u_region;
uniform vec4 u_corners;
uniform vec4 u_capture; // capture origin, inverse blur step, inverse detail step
uniform vec4 u_texels;
uniform vec4 u_detail_texels;
uniform vec4 u_effect; // lens displacement, bevel depth, polish, saturation
uniform vec2 u_layer; // strength and points per pixel
out vec4 pixel;

vec3 scene_at(sampler2D source, vec2 texel, vec4 extent) {
    vec2 uv = clamp(texel, vec2(0.5), extent.xy - 0.5) / extent.zw;
    return texture(source, uv).rgb;
}

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
    float inset = -distance_to_edge;
    float mask = smoothstep(0.0, 1.0, inset);
    if (mask <= 0.0) { discard; }

    float shade = 0.0;
    float specular = 0.0;
    float focus = 0.0;
    if (u_effect.x > 0.0 || u_effect.z > 0.0) {
        float half_side = min(u_region.z, u_region.w);
        float depth = min(u_effect.y, half_side * 0.70);
        float bevel = 1.0 - smoothstep(0.0, depth, inset);
        if (bevel > 0.0) {
            // Reproject in screen space, with a continuous normal between
            // the straight edges and rounded corners.
            float side = smoothstep(-1.25, 1.25, edge.x - edge.y);
            vec2 normal = normalize(vec2(side, 1.0 - side));
            if (arc_distance > 0.001) {
                normal = normalize(mix(normal, max(edge, vec2(0.0)) / arc_distance,
                    smoothstep(0.0, 1.25, arc_distance)));
            }
            normal *= mix(vec2(-1.0), vec2(1.0), step(vec2(0.0), local));
            float displacement = min(u_effect.x, depth * 0.76);
            point += normal * displacement * bevel;
            focus = 0.46 * bevel * smoothstep(0.0, depth * 0.30, displacement);

            float facing = dot(normal, vec2(-0.6, 0.8));
            float key = smoothstep(0.05, 1.0, facing);
            float bounce = smoothstep(0.20, 1.0, -facing);
            // Define the lip in points, independent of bevel depth and DPI.
            float pt = inset * u_layer.y;
            float lip = 1.0 - smoothstep(0.35, 1.30, pt);
            float inner = smoothstep(0.30, 1.15, pt) * (1.0 - smoothstep(1.35, 3.5, pt));
            shade = min(0.24, u_effect.z * inner * (0.105 + 0.065 * bounce));
            specular = min(0.46, u_effect.z
                * (lip * (0.022 + 0.34 * key * key + 0.10 * bounce * bounce)
                    + 0.028 * bevel * key * key));
        }
    }

    vec2 texel = point - u_capture.xy;
    vec3 scene = scene_at(u_texture, texel * u_capture.z, u_texels);
    if (focus > 0.0) {
        vec3 detail = scene_at(u_detail, texel * u_capture.w, u_detail_texels);
        // Keep recognizable color and shape in the bevel while diffusing
        // high-contrast details that would otherwise compete with labels.
        float contrast = length(detail - scene);
        scene = mix(scene, detail, focus * (1.0 - 0.8 * smoothstep(0.08, 0.38, contrast)));
    }
    float luma = dot(scene, vec3(0.2126, 0.7152, 0.0722));
    vec3 color = clamp(mix(vec3(luma), scene, u_effect.w), 0.0, 1.0);
    color = mix(color * (1.0 - shade), vec3(1.0), specular);
    float alpha = mask * u_layer.x;
    pixel = vec4(color * alpha, alpha);
}

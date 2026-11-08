uniform sampler2D u_texture;
uniform vec4 u_texels;
uniform vec2 u_axis;
uniform float u_center;
uniform vec4 u_offsets;
uniform vec4 u_weights;
out vec4 pixel;

vec3 sample_scene(vec2 texel) {
    return texture(u_texture, clamp(texel, vec2(0.5), u_texels.xy - 0.5) / u_texels.zw).rgb;
}

void main() {
    vec2 point = gl_FragCoord.xy;
    vec3 color = sample_scene(point) * u_center;
    for (int tap = 0; tap < 4; ++tap) {
        vec2 offset = u_axis * u_offsets[tap];
        color += (sample_scene(point + offset) + sample_scene(point - offset)) * u_weights[tap];
    }
    pixel = vec4(color, 1.0);
}

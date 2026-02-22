// TAA resolve: reproject at curr_ndc - velocity, 3x3 AABB clamp, YCoCg blend alpha=1/16.

struct VertexOutput {
    @builtin(position) clip: vec4<f32>,
    @location(0) uv: vec2<f32>,
}

@vertex
fn vs(@location(0) pos: vec2<f32>) -> VertexOutput {
    var out: VertexOutput;
    out.clip = vec4<f32>(pos, 0.0, 1.0);
    out.uv = (pos + 1.0) * 0.5;
    return out;
}

fn rgb_to_ycocg(c: vec3<f32>) -> vec3<f32> {
    let y = dot(c, vec3<f32>(0.25, 0.5, 0.25));
    let co = dot(c, vec3<f32>(0.5, 0.0, -0.5));
    let cg = dot(c, vec3<f32>(-0.25, 0.5, -0.25));
    return vec3<f32>(y, co, cg);
}

fn ycocg_to_rgb(y: vec3<f32>) -> vec3<f32> {
    return vec3<f32>(
        y.x + y.y - y.z,
        y.x + y.z,
        y.x - y.y - y.z,
    );
}

@group(0) @binding(0) var color_tex: texture_2d<f32>;
@group(0) @binding(1) var velocity_tex: texture_2d<f32>;
@group(0) @binding(2) var history_tex: texture_2d<f32>;
@group(0) @binding(3) var samp: sampler;

struct TaaOutput {
    @location(0) resolve: vec4<f32>,
    @location(1) history: vec4<f32>,
}

@fragment
fn fs(vin: VertexOutput) -> TaaOutput {
    let texel = textureDimensions(color_tex);
    let uv = vin.uv;
    let velocity = textureSample(velocity_tex, samp, uv).xy;
    let history_uv = uv + velocity * 0.5;
    let current = textureSample(color_tex, samp, uv).rgb;
    let history = textureSample(history_tex, samp, history_uv).rgb;

    let one = 1.0 / vec2<f32>(f32(texel.x), f32(texel.y));
    var min_c = current;
    var max_c = current;
    for (var dy = -1; dy <= 1; dy += 1) {
        for (var dx = -1; dx <= 1; dx += 1) {
            let off = vec2<f32>(f32(dx), f32(dy)) * one;
            let s = textureSample(color_tex, samp, uv + off).rgb;
            min_c = min(min_c, s);
            max_c = max(max_c, s);
        }
    }
    let clamped_history = clamp(history, min_c, max_c);

    let alpha = 1.0 / 16.0;
    let curr_y = rgb_to_ycocg(current);
    let hist_y = rgb_to_ycocg(clamped_history);
    let blended = mix(hist_y, curr_y, alpha);
    let result = ycocg_to_rgb(blended);
    let out_color = vec4<f32>(result, 1.0);
    var out: TaaOutput;
    out.resolve = out_color;
    out.history = out_color;
    return out;
}

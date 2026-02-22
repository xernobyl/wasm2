// Extract bright parts for bloom (threshold + optional scale).

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

@group(0) @binding(0) var resolve_tex: texture_2d<f32>;
@group(0) @binding(1) var samp: sampler;

const THRESHOLD: f32 = 0.4;
const SCALE: f32 = 1.0;

@fragment
fn fs(vin: VertexOutput) -> @location(0) vec4<f32> {
    let c = textureSample(resolve_tex, samp, vin.uv).rgb;
    let lum = dot(c, vec3<f32>(0.299, 0.587, 0.114));
    let scale = max(lum - THRESHOLD, 0.0) / max(lum, 0.001);
    let bright = c * scale * SCALE;
    return vec4<f32>(bright, 1.0);
}

// Kawase-style 4-tap blur (single pass, half-res to half-res).

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

@group(0) @binding(0) var bloom_tex: texture_2d<f32>;
@group(0) @binding(1) var samp: sampler;

@fragment
fn fs(vin: VertexOutput) -> @location(0) vec4<f32> {
    let dims = vec2<f32>(textureDimensions(bloom_tex));
    let one = 1.0 / dims;
    let uv = vin.uv;
    let o = 1.0 * one;
    let c = textureSample(bloom_tex, samp, uv);
    let t = textureSample(bloom_tex, samp, uv + vec2<f32>(0.0, o.y));
    let b = textureSample(bloom_tex, samp, uv - vec2<f32>(0.0, o.y));
    let l = textureSample(bloom_tex, samp, uv - vec2<f32>(o.x, 0.0));
    let r = textureSample(bloom_tex, samp, uv + vec2<f32>(o.x, 0.0));
    let sum = (c + t + b + l + r) * 0.2;
    return vec4<f32>(sum.rgb, 1.0);
}

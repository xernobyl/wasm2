// Fullscreen pass: sample resolve texture and output to swap chain.

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

@fragment
fn fs(vin: VertexOutput) -> @location(0) vec4<f32> {
    return textureSample(resolve_tex, samp, vin.uv);
}

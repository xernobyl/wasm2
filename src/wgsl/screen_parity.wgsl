// Final composite: tonemap(resolve) + bloom, vignette. Output to swap chain.

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
@group(0) @binding(1) var bloom_tex: texture_2d<f32>;
@group(0) @binding(2) var samp: sampler;

@fragment
fn fs(vin: VertexOutput) -> @location(0) vec4<f32> {
    let scene = textureSample(resolve_tex, samp, vin.uv).rgb;
    let bloom = textureSample(bloom_tex, samp, vin.uv).rgb;
    let combined = scene + bloom * 0.4;
    let tonemapped = combined / (combined + vec3<f32>(1.0, 1.0, 1.0));
    let d = length(vin.uv - 0.5) * 1.4;
    let vignette = 1.0 - smoothstep(0.4, 1.0, d);
    let col = tonemapped * vignette;
    return vec4<f32>(col, 1.0);
}

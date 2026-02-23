// 5-tap weighted downsample for bloom mip chain.
// Center 50%, four corners 12.5% each.

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

@group(0) @binding(0) var color_tex: texture_2d<f32>;
@group(0) @binding(1) var samp: sampler;

const RADIUS: f32 = 1.0;

@fragment
fn fs(vin: VertexOutput) -> @location(0) vec4<f32> {
    let dims = vec2<f32>(textureDimensions(color_tex));
    let texel_size = 1.0 / dims;
    let uv = vin.uv;

    let col = textureSample(color_tex, samp, uv).rgb * 0.5
            + textureSample(color_tex, samp, uv - RADIUS * texel_size).rgb * 0.125
            + textureSample(color_tex, samp, uv + RADIUS * texel_size).rgb * 0.125
            + textureSample(color_tex, samp, uv + RADIUS * vec2<f32>(texel_size.x, -texel_size.y)).rgb * 0.125
            + textureSample(color_tex, samp, uv - RADIUS * vec2<f32>(texel_size.x, -texel_size.y)).rgb * 0.125;

    return vec4<f32>(col, 1.0);
}

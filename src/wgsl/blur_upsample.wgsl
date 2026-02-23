// 8-tap tent upsample for bloom mip chain.
// 4 cardinal directions (1/12 each) + 4 diagonal directions (1/6 each).

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
    let half_texel = 0.5 * texel_size;
    let uv = vin.uv;

    let col = textureSample(color_tex, samp, uv + RADIUS * vec2<f32>(-texel_size.x, 0.0)).rgb / 12.0
            + textureSample(color_tex, samp, uv + RADIUS * vec2<f32>(-half_texel.x,  half_texel.y)).rgb / 6.0
            + textureSample(color_tex, samp, uv + RADIUS * vec2<f32>(0.0,  texel_size.y)).rgb / 12.0
            + textureSample(color_tex, samp, uv + RADIUS * vec2<f32>( half_texel.x,  half_texel.y)).rgb / 6.0
            + textureSample(color_tex, samp, uv + RADIUS * vec2<f32>( texel_size.x, 0.0)).rgb / 12.0
            + textureSample(color_tex, samp, uv + RADIUS * vec2<f32>( half_texel.x, -half_texel.y)).rgb / 6.0
            + textureSample(color_tex, samp, uv + RADIUS * vec2<f32>(0.0, -texel_size.y)).rgb / 12.0
            + textureSample(color_tex, samp, uv + RADIUS * vec2<f32>(-half_texel.x, -half_texel.y)).rgb / 6.0;

    return vec4<f32>(col, 1.0);
}

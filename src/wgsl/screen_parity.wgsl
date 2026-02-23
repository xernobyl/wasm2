// Final composite: ACES tonemap, sRGB, weighted scene/bloom blend, starburst, vignette.

const TAU: f32 = 6.283185307179586476925286766559;

struct ScreenUniforms {
    camera_dir: vec3<f32>,
    _pad: f32,
}

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
@group(0) @binding(3) var<uniform> uniforms: ScreenUniforms;

// ACES filmic approximation (Krzysztof Narkowicz)
fn tonemap(v_in: vec3<f32>) -> vec3<f32> {
    let v = v_in * 0.6;
    return clamp((v * (2.51 * v + 0.03)) / (v * (2.43 * v + 0.59) + 0.14), vec3<f32>(0.0), vec3<f32>(1.0));
}

fn srgb(linear: vec3<f32>) -> vec3<f32> {
    let a = 12.92 * linear;
    let b = 1.055 * pow(linear, vec3<f32>(1.0 / 2.4)) - 0.055;
    let c = step(vec3<f32>(0.0031308), linear);
    return mix(a, b, c);
}

fn hash1(p: vec2<f32>) -> f32 {
    return fract(sin(p.x * 0.129898 + p.y * 0.78233) * 43758.5453);
}

fn valueNoise(p: vec2<f32>, s: vec2<f32>) -> f32 {
    let cell = floor(p);
    let sub = p - cell;
    let cube = sub * sub * (3.0 - 2.0 * sub);

    return mix(
        mix(hash1(cell % s), hash1((cell + vec2<f32>(1.0, 0.0)) % s), cube.x),
        mix(hash1((cell + vec2<f32>(0.0, 1.0)) % s), hash1((cell + vec2<f32>(1.0, 1.0)) % s), cube.x),
        cube.y,
    );
}

fn starburst(phi: f32, dir: vec3<f32>) -> f32 {
    let t = dot(vec3<f32>(0.0, 0.0, -1.0), dir) * 0.5 + 0.5;
    return valueNoise(vec2<f32>(phi / TAU * 1000.0, t * 76.54321), vec2<f32>(10000.0, 10000.0)) * 0.25 + 0.75;
}

@fragment
fn fs(vin: VertexOutput) -> @location(0) vec4<f32> {
    let np = vin.uv * 2.0 - 1.0;
    let polar_r = length(np);
    let polar_phi = atan2(np.x, np.y);

    let scene = textureSample(resolve_tex, samp, vin.uv).rgb;
    let bloom = textureSample(bloom_tex, samp, vin.uv).rgb;

    var image = (2.0 / 3.0) * scene + (1.0 / 3.0) * bloom * starburst(polar_phi, uniforms.camera_dir);

    let r = polar_r * 0.5;
    let vignette = pow(1.0 - r * r, 4.0);
    image *= vignette;

    return vec4<f32>(srgb(tonemap(image)), 1.0);
}

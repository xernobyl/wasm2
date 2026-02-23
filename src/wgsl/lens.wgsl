// Lens flare: ghost flares, spectral ring, halo ring.
// Ported from WebGL2 lens.fs.glsl (based on John Chapman's pseudo lens flare).

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

// CIE 1931 XYZ approximation (Sloan)
fn xFit_1931(wave: f32) -> f32 {
    let t1 = (wave - 442.0) * select(0.0374, 0.0624, wave < 442.0);
    let t2 = (wave - 599.8) * select(0.0323, 0.0264, wave < 599.8);
    let t3 = (wave - 501.1) * select(0.0382, 0.0490, wave < 501.1);
    return 0.362 * exp(-0.5 * t1 * t1) + 1.056 * exp(-0.5 * t2 * t2) - 0.065 * exp(-0.5 * t3 * t3);
}

fn yFit_1931(wave: f32) -> f32 {
    let t1 = (wave - 568.8) * select(0.0247, 0.0213, wave < 568.8);
    let t2 = (wave - 530.9) * select(0.0322, 0.0613, wave < 530.9);
    return 0.821 * exp(-0.5 * t1 * t1) + 0.286 * exp(-0.5 * t2 * t2);
}

fn zFit_1931(wave: f32) -> f32 {
    let t1 = (wave - 437.0) * select(0.0278, 0.0845, wave < 437.0);
    let t2 = (wave - 459.0) * select(0.0725, 0.0385, wave < 459.0);
    return 1.217 * exp(-0.5 * t1 * t1) + 0.681 * exp(-0.5 * t2 * t2);
}

fn waveLengthToLinearRGB(w: f32) -> vec3<f32> {
    let XYZ2RGB = mat3x3<f32>(
        vec3<f32>( 3.2406255, -0.9689307,  0.0557101),
        vec3<f32>(-1.5372080,  1.8757561, -0.2040211),
        vec3<f32>(-0.4986286,  0.0415175,  1.0569959),
    );
    let xyz = vec3<f32>(xFit_1931(w), yFit_1931(w), zFit_1931(w));
    return XYZ2RGB * xyz * 0.3968;
}

fn spectralRing(p: vec2<f32>) -> vec3<f32> {
    let rad = length(p);
    let wavelength = mix(700.0, 400.0, smoothstep(0.0, 1.0, rad));
    return waveLengthToLinearRGB(wavelength);
}

fn ghostFlare(uv: vec2<f32>, center: vec2<f32>) -> vec3<f32> {
    var result = vec3<f32>(0.0);
    let delta = center - uv;
    let dist = length(delta);

    for (var i = 0; i <= 3; i++) {
        let scale = f32(i) * 0.2;
        let ghostUV = fract(uv + delta * scale);
        let fade = exp(-dist * scale * 8.0);
        let col = textureSample(bloom_tex, samp, ghostUV).rgb;
        result += col * fade * 0.5;
    }
    return result;
}

fn haloRing(p: vec2<f32>, np: vec2<f32>) -> vec3<f32> {
    let r = length(p) - sqrt(2.0);
    let a = atan2(p.y, p.x);
    let invertedImage = textureSample(bloom_tex, samp, vec2<f32>(r * cos(a), r * sin(a)) * 0.5 + 0.5).rgb;
    let l = length(np);
    let w = max(1.0 - pow(abs((l - 0.6666666666) * 5.0), 3.0), 0.0);
    return invertedImage * w;
}

@fragment
fn fs(vin: VertexOutput) -> @location(0) vec4<f32> {
    let center = vec2<f32>(0.5);
    let uv = vec2<f32>(1.0) - vin.uv;
    let np = vin.uv * 2.0 - 1.0;
    let p = np;

    let spectral = spectralRing(np);

    let halo = 0.1 * textureSample(bloom_tex, samp, center).rgb;
    let ghosts = 0.1 * ghostFlare(uv, center);
    let ring = 0.1 * haloRing(p, np);

    var col = textureSample(bloom_tex, samp, vin.uv).rgb;
    col += ghosts * (2.0 / 3.0 * spectral + 1.0 / 3.0);
    col += ring * (0.5 * spectral + 0.5);
    col += halo * spectral;

    return vec4<f32>(col, 1.0);
}

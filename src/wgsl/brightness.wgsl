// Combined brightness extraction + 5-tap downsample (first bloom mip pass).
// Ported from WebGL2 blur_brightness.fs.glsl + blur.glsl + openSimplex2.glsl.

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

// ---- OpenSimplex2 (K.jpg's Re-oriented 4-Point BCC) ----

fn permute_v4(t: vec4<f32>) -> vec4<f32> { return t * (t * 34.0 + 133.0); }

fn grad_noise(hash_val: f32) -> vec3<f32> {
    let cube = (floor(hash_val / vec3<f32>(1.0, 2.0, 4.0)) % 2.0) * 2.0 - 1.0;
    var cuboct = cube;
    let idx = i32(hash_val / 16.0);
    if (idx == 0) { cuboct.x = 0.0; }
    else if (idx == 1) { cuboct.y = 0.0; }
    else { cuboct.z = 0.0; }
    let type_val = floor(hash_val / 8.0) % 2.0;
    let rhomb = (1.0 - type_val) * cube + type_val * (cuboct + cross(cube, cuboct));
    var grad_result = cuboct * 1.22474487139 + rhomb;
    grad_result *= (1.0 - 0.042942436724648037 * type_val) * 32.80201376986577;
    return grad_result;
}

fn openSimplex2Base(X: vec3<f32>) -> vec4<f32> {
    let v1 = round(X);
    let d1 = X - v1;
    let score1 = abs(d1);
    let dir1 = select(vec3<f32>(0.0), vec3<f32>(1.0), max(score1.yzx, score1.zxy) <= score1);
    let v2 = v1 + dir1 * sign(d1);
    let d2 = X - v2;
    let X2 = X + 144.5;
    let v3 = round(X2);
    let d3 = X2 - v3;
    let score2 = abs(d3);
    let dir2 = select(vec3<f32>(0.0), vec3<f32>(1.0), max(score2.yzx, score2.zxy) <= score2);
    let v4 = v3 + dir2 * sign(d3);
    let d4 = X2 - v4;
    var hashes = permute_v4(vec4<f32>(v1.x, v2.x, v3.x, v4.x) % 289.0);
    hashes = permute_v4((hashes + vec4<f32>(v1.y, v2.y, v3.y, v4.y)) % 289.0);
    hashes = permute_v4((hashes + vec4<f32>(v1.z, v2.z, v3.z, v4.z)) % 289.0) % 48.0;
    let a = max(0.5 - vec4<f32>(dot(d1, d1), dot(d2, d2), dot(d3, d3), dot(d4, d4)), vec4<f32>(0.0));
    let aa = a * a;
    let aaaa = aa * aa;
    let g1 = grad_noise(hashes.x);
    let g2 = grad_noise(hashes.y);
    let g3 = grad_noise(hashes.z);
    let g4 = grad_noise(hashes.w);
    let extrapolations = vec4<f32>(dot(d1, g1), dot(d2, g2), dot(d3, g3), dot(d4, g4));
    let mat_d = mat4x3<f32>(d1, d2, d3, d4);
    let mat_g = mat4x3<f32>(g1, g2, g3, g4);
    let derivative = -8.0 * (mat_d * (aa * a * extrapolations)) + (mat_g * aaaa);
    return vec4<f32>(derivative, dot(aaaa, extrapolations));
}

fn openSimplex2_ImproveXY(X: vec3<f32>) -> vec4<f32> {
    let orthonormalMap = mat3x3<f32>(
        vec3<f32>( 0.788675134594813, -0.211324865405187, -0.577350269189626),
        vec3<f32>(-0.211324865405187,  0.788675134594813, -0.577350269189626),
        vec3<f32>( 0.577350269189626,  0.577350269189626,  0.577350269189626),
    );
    let result = openSimplex2Base(orthonormalMap * X);
    return vec4<f32>(result.xyz * orthonormalMap, result.w);
}

// ---- Brightness + Downsample ----

const KNEE: f32 = 0.5;

fn brightnessPass(color: vec3<f32>, threshold: f32) -> vec3<f32> {
    let lum = dot(color, vec3<f32>(0.2126, 0.7152, 0.0722)) - threshold;
    let softness = clamp(lum / KNEE, 0.0, 1.0);
    return color * softness;
}

fn sampleBright(uv: vec2<f32>, threshold: f32) -> vec3<f32> {
    return brightnessPass(textureSample(color_tex, samp, uv).rgb, threshold);
}

const RADIUS: f32 = 1.0;

@fragment
fn fs(vin: VertexOutput) -> @location(0) vec4<f32> {
    let dims = vec2<f32>(textureDimensions(color_tex));
    let texel_size = 1.0 / dims;
    let uv = vin.uv;

    let noise_val = openSimplex2_ImproveXY(vec3<f32>(uv * vec2<f32>(16.0, 9.0) / 2.0, 0.46546)).w * 0.5 + 0.5;
    let threshold = 1.0 - pow(noise_val, 4.0);

    let col = sampleBright(uv, threshold) * 0.5
            + sampleBright(uv - RADIUS * texel_size, threshold) * 0.125
            + sampleBright(uv + RADIUS * texel_size, threshold) * 0.125
            + sampleBright(uv + RADIUS * vec2<f32>(texel_size.x, -texel_size.y), threshold) * 0.125
            + sampleBright(uv - RADIUS * vec2<f32>(texel_size.x, -texel_size.y), threshold) * 0.125;

    return vec4<f32>(col, 1.0);
}

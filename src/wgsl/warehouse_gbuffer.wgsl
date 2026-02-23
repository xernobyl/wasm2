// Full warehouse scene for G-buffer pass.
// color @0, velocity @1, depth (reversed Z).

struct WarehouseUniforms {
    inverse_view: mat4x4<f32>,
    view_projection: mat4x4<f32>,
    inverse_view_projection: mat4x4<f32>,
    inverse_projection: mat4x4<f32>,
    view_projection_no_jitter: mat4x4<f32>,
    previous_view_projection_no_jitter: mat4x4<f32>,
    time: f32,
    _pad0: f32,
    resolution: vec3<f32>,
    viewport_origin: vec2<f32>,
    viewport_size: vec2<f32>,
    _pad1: vec2<f32>,
    fov: f32,
}

@group(0) @binding(0) var<uniform> u: WarehouseUniforms;

struct VertexOutput {
    @builtin(position) clip: vec4<f32>,
    @location(0) ndc: vec2<f32>,
}

@vertex
fn vs(@location(0) pos: vec2<f32>) -> VertexOutput {
    var out: VertexOutput;
    out.clip = vec4<f32>(pos, 0.0, 1.0);
    out.ndc = pos;
    return out;
}

// ---------- common ----------

const pi: f32 = 3.1415926535897932384626433832795;
const tau: f32 = 6.283185307179586476925286766559;
const EPSILON: f32 = 2e-24;

fn saturate_f(x: f32) -> f32 { return clamp(x, 0.0, 1.0); }

struct BaseMaterial { albedo: vec3<f32>, roughness: f32, metallic: f32, }

fn palette(t: f32, a: vec3<f32>, b: vec3<f32>, c: vec3<f32>, d: vec3<f32>) -> vec3<f32> {
    return a + b * cos(2.0 * pi * (c * t + d));
}

fn min4(a: f32, b: f32, c: f32, d: f32) -> f32 { return min(a, min(b, min(c, d))); }

fn lightingModel(V: vec3<f32>, N: vec3<f32>, L: vec3<f32>, albedo: vec3<f32>, metallic: f32, roughness: f32) -> vec3<f32> {
    let H = normalize(V + L);
    let a = roughness * roughness;
    let a2 = a * a;
    let NdotH = max(dot(N, H), 0.0);
    let NDF = a2 / (pi * pow(NdotH * NdotH * (a2 - 1.0) + 1.0, 2.0) + EPSILON);
    let NdotV = max(dot(N, V), 0.0);
    let NdotL = max(dot(N, L), 0.0);
    var k = (roughness + 1.0);
    k = (k * k) / 8.0;
    let G_V = NdotV / (NdotV * (1.0 - k) + k);
    let G_L = NdotL / (NdotL * (1.0 - k) + k);
    let G = G_V * G_L;
    let HdotV = max(dot(H, V), 0.0);
    let F0 = mix(vec3<f32>(0.04), albedo, metallic);
    let F = F0 + (1.0 - F0) * pow(1.0 - HdotV, 5.0);
    let spec = (NDF * G * F) / (4.0 * NdotV * NdotL + EPSILON);
    let kD = (1.0 - F) * (1.0 - metallic);
    let diffuse = (albedo / pi) * kD;
    return (diffuse + spec) * NdotL;
}

// ---------- hash ----------

const hashMagic: vec4<f32> = vec4<f32>(0.1031, 0.1030, 0.0973, 0.1099);
const hashMagic2: f32 = 33.33;

fn hash1_v3(p3_in: vec3<f32>) -> f32 {
    var p3 = fract(p3_in * hashMagic.x);
    p3 += dot(p3, p3.zyx + 31.32);
    return fract((p3.x + p3.y) * p3.z);
}

fn hash4_v3(p_in: vec3<f32>) -> vec4<f32> {
    var p4 = fract(vec4<f32>(p_in.x, p_in.y, p_in.z, p_in.x) * hashMagic);
    p4 += dot(p4, p4.wzxy + hashMagic2);
    return fract((p4.xxyz + p4.yzzw) * p4.zywx);
}

fn hash4_v4(p4_in: vec4<f32>) -> vec4<f32> {
    var p4 = fract(p4_in * hashMagic);
    p4 += dot(p4, p4.wzxy + hashMagic2);
    return fract((p4.xxyz + p4.yzzw) * p4.zywx);
}

// ---------- openSimplex2 ----------

fn permute_v4(t: vec4<f32>) -> vec4<f32> { return t * (t * 34.0 + 133.0); }

fn grad(hash_val: f32) -> vec3<f32> {
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
    let g1 = grad(hashes.x);
    let g2 = grad(hashes.y);
    let g3 = grad(hashes.z);
    let g4 = grad(hashes.w);
    let extrapolations = vec4<f32>(dot(d1, g1), dot(d2, g2), dot(d3, g3), dot(d4, g4));
    let mat_d = mat4x3<f32>(d1, d2, d3, d4);
    let mat_g = mat4x3<f32>(g1, g2, g3, g4);
    let derivative = -8.0 * (mat_d * (aa * a * extrapolations)) + (mat_g * aaaa);
    return vec4<f32>(derivative, dot(aaaa, extrapolations));
}

fn openSimplex2_Conventional(X: vec3<f32>) -> vec4<f32> {
    let rotated = dot(X, vec3<f32>(2.0 / 3.0)) - X;
    let result = openSimplex2Base(rotated);
    return vec4<f32>(dot(result.xyz, vec3<f32>(2.0 / 3.0)) - result.xyz, result.w);
}

// ---------- distance ----------

fn sdBox(p: vec3<f32>, b: vec3<f32>) -> f32 {
    let q = abs(p) - b;
    return length(max(q, vec3<f32>(0.0))) + min(max(q.x, max(q.y, q.z)), 0.0);
}

fn sdSphere(p: vec3<f32>, r: f32) -> f32 { return length(p) - r; }

fn sdVerticalCapsule(p_in: vec3<f32>, h: f32, r: f32) -> f32 {
    var p = p_in;
    p.y -= clamp(p.y, 0.0, h);
    return length(p) - r;
}

fn opRep(p: vec3<f32>, c: vec3<f32>) -> vec3<f32> {
    let q = (p + 0.5 * c) % c - 0.5 * c;
    return q;
}

fn opSmoothUnion(d1: f32, d2: f32, k: f32) -> f32 {
    let h = clamp(0.5 + 0.5 * (d2 - d1) / k, 0.0, 1.0);
    return mix(d2, d1, h) - k * h * (1.0 - h);
}

fn opSmoothSubtraction(d1: f32, d2: f32, k: f32) -> f32 {
    let h = clamp(0.5 - 0.5 * (d2 + d1) / k, 0.0, 1.0);
    return mix(d2, -d1, h) + k * h * (1.0 - h);
}

// ---------- warehouse scene ----------

const NUM_LIGHTS: i32 = 2;
const maxIterations: u32 = 256u;
const marchFar: f32 = 100.0;
const marchNear: f32 = 0.0;
const holeRepeat: f32 = 0.5;
const holeRadius: f32 = 1.0 / 32.0;

var<private> halfPixelScale: f32;
var<private> objectId: u32;
var<private> lightPos: array<vec3<f32>, 2>;

fn rdSph(p: vec3<f32>, rid: vec4<f32>) -> f32 {
    let h = hash4_v4(rid);
    let radius = h.w * holeRadius;
    let center = (2.0 * h.xyz - 1.0) * (0.5 * holeRepeat - radius);
    return distance(p, center) - radius;
}

fn warehouse_sdf(p: vec3<f32>, include_lights: bool, object: ptr<function, u32>) -> f32 {
    var objs: array<f32, 3>;
    let pillar_pos = opRep(p + vec3<f32>(-15.0, -2.5, 5.0), vec3<f32>(15.0, 0.0, 20.0));
    let main_walls = -sdBox(p + vec3<f32>(0.0, -2.5, 0.0), vec3<f32>(20.0, 5.0, 40.0));
    let pillars = sdBox(pillar_pos, vec3<f32>(0.5, 5.0, 0.5));
    let bars1 = sdBox(opRep(p + vec3<f32>(0.0, -7.0, 0.0), vec3<f32>(15.0, 1000.0, 0.0)), vec3<f32>(0.5, 0.5, 40.0));
    let bars2 = sdBox(opRep(p + vec3<f32>(0.0, -7.0, 5.0), vec3<f32>(1000.0, 1000.0, 20.0)), vec3<f32>(20.0, 0.5, 0.5));
    let bars = min(bars1, bars2);
    var bars_and_pillars = opSmoothUnion(bars, pillars, 0.25);

    var hole0 = 1e20;
    for (var i = 1.0; i < 4.0; i += 1.0) {
        let id = round(p / holeRepeat);
        let offset = sign(p - holeRepeat * id);
        var d = 1e20;
        for (var k = 0; k < 2; k++) {
            for (var j = 0; j < 2; j++) {
                for (var i2 = 0; i2 < 2; i2++) {
                    let rid = id + vec3<f32>(f32(i2), f32(j), f32(k)) * offset;
                    let r = p - holeRepeat * rid;
                    d = min(d, rdSph(r, vec4<f32>(rid, f32(i2))));
                }
            }
        }
        bars_and_pillars = opSmoothSubtraction(d, bars_and_pillars, 4.0 * holeRadius);
    }

    let rebar = min4(
        sdVerticalCapsule(pillar_pos + vec3<f32>(-0.5 / 1.5, 10.0, 0.5 / 1.5), 100.0, 0.02),
        sdVerticalCapsule(pillar_pos + vec3<f32>(0.5 / 1.5, 10.0, 0.5 / 1.5), 100.0, 0.02),
        sdVerticalCapsule(pillar_pos + vec3<f32>(-0.5 / 1.5, 10.0, -0.5 / 1.5), 100.0, 0.02),
        sdVerticalCapsule(pillar_pos + vec3<f32>(0.5 / 1.5, 10.0, -0.5 / 1.5), 100.0, 0.02)
    );

    bars_and_pillars = opSmoothSubtraction(hole0, bars_and_pillars, 1.0 / 32.0);
    let room = opSmoothUnion(main_walls, bars_and_pillars, 1.0 / 16.0);

    let pipes = min(
        sdVerticalCapsule(p.xzy - vec3<f32>(18.0, -50.0, 6.0), 200.0, 0.10),
        sdVerticalCapsule(p.xzy - vec3<f32>(18.0, -50.0, 5.6), 200.0, 0.10)
    );

    objs[0] = room;
    objs[1] = rebar;
    objs[2] = pipes;

    var min_dist = objs[0];
    *object = 1u;
    for (var i = 1u; i < 3u; i++) {
        if (objs[i] < min_dist) {
            min_dist = objs[i];
            *object = i + 1u;
        }
    }

    if (include_lights) {
        for (var i = 0; i < NUM_LIGHTS; i++) {
            let d = sdSphere(p - lightPos[i], 0.125);
            if (d < min_dist) {
                min_dist = d;
                *object = u32(i + 4);
            }
        }
    }
    return min_dist;
}

fn scene_dist(p: vec3<f32>) -> f32 {
    var obj: u32;
    let dist = warehouse_sdf(p, true, &obj);
    objectId = obj;
    return dist;
}

fn sceneNoLights(p: vec3<f32>) -> f32 {
    var obj: u32;
    return warehouse_sdf(p, false, &obj);
}

fn sceneNormal(p: vec3<f32>, d: f32) -> vec3<f32> {
    let k = vec2<f32>(1.0, -1.0);
    let sb = vec4<f32>(2.0, tau, 1.0, pi);
    let h = halfPixelScale * d;
    let r = hash4_v3(p) * sb.xxxy - sb.zzzw;
    let r_xyz = normalize(r.xyz);
    let pt0 = k.xyy;
    let pt1 = k.yyx;
    let pt2 = k.yxy;
    let pt3 = k.xxx;
    return normalize(
        pt0 * scene_dist(p + h * pt0) +
        pt1 * scene_dist(p + h * pt1) +
        pt2 * scene_dist(p + h * pt2) +
        pt3 * scene_dist(p + h * pt3)
    );
}

fn calcSoftshadow(ro: vec3<f32>, rd: vec3<f32>, error: f32, mint: f32, tmax: f32, w: f32) -> f32 {
    var res = 1.0;
    var t = mint;
    var ph = 1e10;
    for (var i = 0u; i < 32u; i++) {
        let h = sceneNoLights(ro + rd * t);
        let y = h * h / (2.0 * ph);
        let d = sqrt(h * h - y * y);
        res = min(res, d / (w * max(0.0, t - y)));
        ph = h;
        t += h;
        if (res <= error || t > tmax) { break; }
    }
    res = saturate_f(res);
    return res * res * (3.0 - 2.0 * res);
}

fn render(uv: vec2<f32>, rayOrigin: vec3<f32>, cx: vec3<f32>, cy: vec3<f32>, cz: vec3<f32>, zoom: f32, pos: ptr<function, vec3<f32>>) -> vec3<f32> {
    let rayDir = normalize(uv.x * cx + uv.y * cy - zoom * cz);
    var rayPos = rayOrigin;

    objectId = 0u;
    let noiz = halfPixelScale * (2.0 * hash1_v3(vec3<f32>(uv, u.time)) - 1.0);
    var totalDistance = marchNear + noiz;

    for (var it = 0u; it < maxIterations; it++) {
        let stepSize = scene_dist(rayPos);
        totalDistance += stepSize;
        rayPos = rayOrigin + totalDistance * rayDir;
        if (stepSize <= halfPixelScale * totalDistance) { break; }
        if (totalDistance >= marchFar) {
            objectId = 0u;
            break;
        }
    }

    var lightColor: array<vec3<f32>, 2>;
    lightColor[0] = vec3<f32>(1.0, 0.95, 0.8) * 35.0;
    lightColor[1] = vec3<f32>(0.8, 0.9, 1.0) * 25.0;

    if (objectId == 0u) { return vec3<f32>(1.0, 0.0, 0.0); }
    if (objectId == 4u) { return lightColor[0] / (0.125 * 0.125); }
    if (objectId == 5u) { return lightColor[1] / (0.125 * 0.125); }

    var material: BaseMaterial;
    if (objectId == 1u) {
        let n = openSimplex2_Conventional(rayPos * 0.5).w * 0.5 + 0.5;
        material.albedo = vec3<f32>(1.0);
        material.roughness = n;
        material.metallic = 0.0;
    } else if (objectId == 2u) {
        material.albedo = vec3<f32>(185.0 / 255.0, 71.0 / 255.0, 0.0);
        material.roughness = 0.25;
        material.metallic = 0.25;
    } else if (objectId == 3u) {
        material.albedo = vec3<f32>(0.0, 0.0, 1.0);
        material.roughness = 0.75;
        material.metallic = 0.0;
    }

    let N = sceneNormal(rayPos, totalDistance);
    var color = vec3<f32>(0.0);
    for (var i = 0; i < NUM_LIGHTS; i++) {
        let L = normalize(lightPos[i] - rayPos);
        let d = distance(rayPos, lightPos[i]);
        var lightValue = lightColor[i] / (d * d);
        lightValue *= calcSoftshadow(rayPos, L, totalDistance * halfPixelScale * 2.0, 0.25, d, 0.01);
        color += lightingModel(-rayDir, N, L, material.albedo, material.metallic, material.roughness) * lightValue;
    }

    *pos = rayPos;
    return color;
}

// ---------- fragment ----------

struct FragmentOutput {
    @location(0) color: vec4<f32>,
    @location(1) velocity: vec2<f32>,
    @builtin(frag_depth) depth: f32,
}

@fragment
fn fs(vin: VertexOutput) -> FragmentOutput {
    lightPos[0] = vec3<f32>(cos(u.time * tau / 4.0) * 2.0, sin(u.time * tau / 2.0) * 1.0, sin(u.time * tau / 4.0) * 2.0);
    lightPos[1] = vec3<f32>(cos(u.time * tau / 2.0) * 4.0, -1.0, sin(u.time * tau / 2.0) * 4.0);

    let thf = tan(u.fov * 0.5);
    let zoom = 1.0 / thf;
    halfPixelScale = thf * u.resolution.z;

    let uv = vec2<f32>(vin.clip.x * 2.0 - u.resolution.x, u.resolution.y - vin.clip.y * 2.0);
    let uv_scaled = uv * u.resolution.z;

    let cx = u.inverse_view[0].xyz;
    let cy = u.inverse_view[1].xyz;
    let cz = u.inverse_view[2].xyz;
    let rayOrigin = u.inverse_view[3].xyz;

    var pos: vec3<f32>;
    let outColor = render(uv_scaled, rayOrigin, cx, cy, cz, zoom, &pos);

    var out: FragmentOutput;
    out.velocity = vec2<f32>(0.0, 0.0);

    if (objectId == 0u) {
        out.color = vec4<f32>(outColor, 1.0);
        out.depth = 0.0;
        return out;
    }

    let world_pos = vec4<f32>(pos, 1.0);
    let curr_clip = u.view_projection_no_jitter * world_pos;
    let prev_clip = u.previous_view_projection_no_jitter * world_pos;
    out.velocity = prev_clip.xy / prev_clip.w - curr_clip.xy / curr_clip.w;

    let clip_pos = u.view_projection * world_pos;
    out.depth = clip_pos.z / clip_pos.w;
    out.color = vec4<f32>(outColor, 1.0);
    return out;
}

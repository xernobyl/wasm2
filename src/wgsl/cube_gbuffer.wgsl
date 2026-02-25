// Half-cube for G-buffer. Same storage buffer and octant-flip as cube.wgsl.

struct CubeUniforms {
    view_projection: mat4x4<f32>,
    view_projection_no_jitter: mat4x4<f32>,
    previous_view_projection_no_jitter: mat4x4<f32>,
    camera_position: vec3<f32>,
    _pad: f32,
}

@group(0) @binding(0) var<uniform> u: CubeUniforms;
@group(0) @binding(1) var<storage, read> instances: array<vec4<f32>>;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @builtin(instance_index) instance_index: u32,
}

struct VertexOutput {
    @builtin(position) clip: vec4<f32>,
    @location(0) world_pos: vec3<f32>,
    @location(1) local_mirrored_pos: vec3<f32>,
    @location(2) @interpolate(flat) instance_index: u32,
}

fn flip_by_octant(pos: vec3<f32>, sign: vec3<f32>) -> vec3<f32> {
    return 0.5 * (sign * (2.0 * pos - 1.0) + 1.0);
}

@vertex
fn vs(in: VertexInput) -> VertexOutput {
    let pos_scale = instances[in.instance_index];
    let cube_pos = pos_scale.xyz;
    let scale = pos_scale.w;
    let to_camera = u.camera_position - cube_pos;
    let s = vec3<f32>(sign(to_camera.x), sign(to_camera.y), sign(to_camera.z));
    let sign = vec3<f32>(
        select(1.0, s.x, abs(s.x) > 0.001),
        select(1.0, s.y, abs(s.y) > 0.001),
        select(1.0, s.z, abs(s.z) > 0.001),
    );
    let flipped = flip_by_octant(in.position, sign);
    let world_pos = cube_pos - scale * flipped;

    var out: VertexOutput;
    out.clip = u.view_projection * vec4<f32>(world_pos, 1.0);
    if (sign.x * sign.y * sign.z < 0.0) {
        out.clip.x = -out.clip.x;
        out.clip.y = -out.clip.y;
    }
    out.world_pos = world_pos;
    out.local_mirrored_pos = flipped;
    out.instance_index = in.instance_index;
    return out;
}

fn face_normal_from_local_pos(p: vec3<f32>) -> vec3<f32> {
    let ax = abs(p.x - 0.5);
    let ay = abs(p.y - 0.5);
    let az = abs(p.z - 0.5);
    if (ax >= ay && ax >= az) {
        return vec3<f32>(select(-1.0, 1.0, p.x >= 0.5), 0.0, 0.0);
    }
    if (ay >= az) {
        return vec3<f32>(0.0, select(-1.0, 1.0, p.y >= 0.5), 0.0);
    }
    return vec3<f32>(0.0, 0.0, select(-1.0, 1.0, p.z >= 0.5));
}

fn instance_index_to_color(idx: u32) -> vec3<f32> {
    let t = f32(idx);
    let r = fract(sin(t * 12.9898) * 43758.5453);
    let g = fract(sin(t * 78.233 + 12.9898) * 43758.5453);
    let b = fract(sin(t * 45.164 + 78.233) * 43758.5453);
    return vec3<f32>(r, g, b);
}

struct FragmentOutput {
    @location(0) color: vec4<f32>,
    @location(1) velocity: vec2<f32>,
}

@fragment
fn fs(in: VertexOutput) -> FragmentOutput {
    let world_pos = vec4<f32>(in.world_pos, 1.0);
    let curr_clip = u.view_projection_no_jitter * world_pos;
    let prev_clip = u.previous_view_projection_no_jitter * world_pos;
    let curr_ndc = curr_clip.xy / curr_clip.w;
    let prev_ndc = prev_clip.xy / prev_clip.w;
    let velocity = prev_ndc - curr_ndc;

    let face_normal = face_normal_from_local_pos(in.local_mirrored_pos);
    let light_dir = normalize(vec3<f32>(1.0, 2.0, 1.0));
    let ndotl = max(dot(face_normal, light_dir), 0.0);
    let diffuse = 0.4 + 0.5 * ndotl;
    let base = instance_index_to_color(in.instance_index);
    let col = (0.35 + 0.65 * base) * diffuse;

    var out: FragmentOutput;
    out.color = vec4<f32>(col, 1.0);
    out.velocity = velocity;
    return out;
}

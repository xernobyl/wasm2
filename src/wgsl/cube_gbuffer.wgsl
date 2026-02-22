// Half-cube for G-buffer. Same octant-flip as cube.wgsl so 3 faces look like a full cube.

struct CubeUniforms {
    view_projection: mat4x4<f32>,
    view_projection_no_jitter: mat4x4<f32>,
    previous_view_projection_no_jitter: mat4x4<f32>,
    camera_position: vec3<f32>,
    _pad: f32,
}

@group(0) @binding(0) var<uniform> u: CubeUniforms;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(2) instance_col0: vec4<f32>,
    @location(3) instance_col1: vec4<f32>,
    @location(4) instance_col2: vec4<f32>,
    @location(5) instance_col3: vec4<f32>,
}

struct VertexOutput {
    @builtin(position) clip: vec4<f32>,
    @location(0) world_pos: vec3<f32>,
    @location(1) world_normal: vec3<f32>,
}

fn cube_normal_from_position(p: vec3<f32>) -> vec3<f32> {
    if (length(p) < 0.001) {
        return vec3<f32>(1.0 / sqrt(3.0), 1.0 / sqrt(3.0), 1.0 / sqrt(3.0));
    }
    return normalize(p);
}

fn flip_by_octant(pos: vec3<f32>, sign: vec3<f32>) -> vec3<f32> {
    return 0.5 * (sign * (2.0 * pos - 1.0) + 1.0);
}

@vertex
fn vs(in: VertexInput) -> VertexOutput {
    let model = mat4x4<f32>(in.instance_col0, in.instance_col1, in.instance_col2, in.instance_col3);
    let instance_origin = model[3].xyz;
    let to_camera = u.camera_position - instance_origin;
    let s = vec3<f32>(sign(to_camera.x), sign(to_camera.y), sign(to_camera.z));
    let sign = vec3<f32>(select(1.0, s.x, abs(s.x) > 0.001), select(1.0, s.y, abs(s.y) > 0.001), select(1.0, s.z, abs(s.z) > 0.001));
    let flipped = flip_by_octant(in.position, sign);
    let world_pos = model * vec4<f32>(flipped, 1.0);
    let n_local = cube_normal_from_position(flipped);
    let world_normal = normalize((model * vec4<f32>(n_local, 0.0)).xyz);
    var out: VertexOutput;
    out.clip = u.view_projection * world_pos;
    out.world_pos = world_pos.xyz;
    out.world_normal = world_normal;
    return out;
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

    let light_dir = normalize(vec3<f32>(1.0, 2.0, 1.0));
    let ndotl = max(dot(in.world_normal, light_dir), 0.0);
    let diffuse = 0.4 + 0.5 * ndotl;
    let col = vec3<f32>(0.6, 0.5, 0.4) * diffuse;

    var out: FragmentOutput;
    out.color = vec4<f32>(col, 1.0);
    out.velocity = velocity;
    return out;
}

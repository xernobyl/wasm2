// Chunk mesh: greedy-meshed quads. Vertex position + normal, single view_projection.
// Same shading as cube (simple diffuse).

struct ChunkUniforms {
    view_projection: mat4x4<f32>,
}

@group(0) @binding(0) var<uniform> u: ChunkUniforms;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
}

struct VertexOutput {
    @builtin(position) clip: vec4<f32>,
    @location(0) world_normal: vec3<f32>,
}

@vertex
fn vs(in: VertexInput) -> VertexOutput {
    let world_pos = vec4<f32>(in.position, 1.0);
    var out: VertexOutput;
    out.clip = u.view_projection * world_pos;
    out.world_normal = normalize(in.normal);
    return out;
}

@fragment
fn fs(in: VertexOutput) -> @location(0) vec4<f32> {
    let light_dir = normalize(vec3<f32>(1.0, 2.0, 1.0));
    let ndotl = max(dot(in.world_normal, light_dir), 0.0);
    let diffuse = 0.4 + 0.5 * ndotl;
    let col = vec3<f32>(0.5, 0.6, 0.5) * diffuse;
    return vec4<f32>(col, 1.0);
}

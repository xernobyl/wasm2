// Particles: instanced quads (one quad per particle). Vertex: quad corner offset; instance: position xyz.

struct ParticleUniforms {
    view_projection: mat4x4<f32>,
    point_scale: f32,
}

@group(0) @binding(0) var<uniform> u: ParticleUniforms;

struct VertexInput {
    @location(0) offset: vec2<f32>,
    @location(1) instance_pos: vec3<f32>,
}

struct VertexOutput {
    @builtin(position) clip: vec4<f32>,
}

@vertex
fn vs(in: VertexInput) -> VertexOutput {
    let world_pos = in.instance_pos + vec3<f32>(in.offset.x * u.point_scale, in.offset.y * u.point_scale, 0.0);
    var out: VertexOutput;
    out.clip = u.view_projection * vec4<f32>(world_pos, 1.0);
    return out;
}

@fragment
fn fs(in: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(1.0, 0.9, 0.6, 1.0);
}

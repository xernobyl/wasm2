//! Particle system: CPU-updated positions, WebGPU draw as instanced quads.

use glam::Mat4;
use wgpu::RenderPass;
use wgpu::util::DeviceExt;

const PARTICLES_WGSL: &str = include_str!("wgsl/particles.wgsl");

/// Quad vertex offsets (2 triangles, 4 vertices).
const QUAD_OFFSETS: [[f32; 2]; 4] = [
    [-1.0, -1.0],
    [1.0, -1.0],
    [1.0, 1.0],
    [-1.0, 1.0],
];

const QUAD_INDICES: [u16; 6] = [0, 1, 2, 0, 2, 3];

const MAX_PARTICLES: usize = 2048;

/// Particle system: positions updated on CPU, drawn as small quads.
#[derive(Debug)]
pub struct Particles {
    positions: Vec<[f32; 3]>,
    gpu: Option<ParticlesGpu>,
}

/// WebGPU pipeline and buffers for particles (instanced quads).
#[derive(Debug)]
pub struct ParticlesGpu {
    pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    instance_buffer: wgpu::Buffer,
    uniform_buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
    queue: wgpu::Queue,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct ParticleUniforms {
    view_projection: [f32; 16],
    point_scale: f32,
}

unsafe impl bytemuck::Pod for ParticleUniforms {}
unsafe impl bytemuck::Zeroable for ParticleUniforms {}

impl Particles {
    pub fn new() -> Self {
        Self {
            positions: Vec::new(),
            gpu: None,
        }
    }

    /// Set particle positions (xyz). Call [`update_buffer`](Self::update_buffer) after to upload.
    pub fn set_positions(&mut self, positions: &[[f32; 3]]) {
        self.positions.clear();
        self.positions.extend(positions.iter().take(MAX_PARTICLES));
    }

    /// Upload to GPU and create pipeline. Call when WebGPU is ready.
    pub fn upload_to_gpu(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        color_format: wgpu::TextureFormat,
    ) {
        self.gpu = Some(ParticlesGpu::new(device, queue, color_format));
    }

    /// Update GPU instance buffer from current positions.
    pub fn update_buffer(&self) {
        if let Some(ref g) = self.gpu {
            g.upload_positions(&self.positions);
        }
    }

    /// Draw particles. Call [`update_buffer`](Self::update_buffer) first if positions changed.
    pub fn draw(
        &self,
        pass: Option<&mut RenderPass<'_>>,
        view_projection: Mat4,
        point_scale: f32,
    ) {
        if self.positions.is_empty() {
            return;
        }
        if let (Some(ref g), Some(p)) = (&self.gpu, pass) {
            g.draw(p, view_projection, point_scale, self.positions.len() as u32);
        }
    }
}

impl Default for Particles {
    fn default() -> Self {
        Self::new()
    }
}

impl ParticlesGpu {
    fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        color_format: wgpu::TextureFormat,
    ) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("particles"),
            source: wgpu::ShaderSource::Wgsl(PARTICLES_WGSL.into()),
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("particles_bind_layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("particles_layout"),
            bind_group_layouts: &[&bind_group_layout],
            immediate_size: 0,
        });

        let uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("particles_uniform"),
            size: std::mem::size_of::<ParticleUniforms>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("particles_quad_vertices"),
            contents: bytemuck::cast_slice(&QUAD_OFFSETS),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("particles_quad_indices"),
            contents: bytemuck::cast_slice(&QUAD_INDICES),
            usage: wgpu::BufferUsages::INDEX,
        });

        let instance_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("particles_instances"),
            size: (MAX_PARTICLES * 3 * 4) as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("particles_bind_group"),
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            }],
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("particles"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs"),
                buffers: &[
                    wgpu::VertexBufferLayout {
                        array_stride: 8,
                        step_mode: wgpu::VertexStepMode::Vertex,
                        attributes: &[wgpu::VertexAttribute {
                            offset: 0,
                            shader_location: 0,
                            format: wgpu::VertexFormat::Float32x2,
                        }],
                    },
                    wgpu::VertexBufferLayout {
                        array_stride: 12,
                        step_mode: wgpu::VertexStepMode::Instance,
                        attributes: &[wgpu::VertexAttribute {
                            offset: 0,
                            shader_location: 1,
                            format: wgpu::VertexFormat::Float32x3,
                        }],
                    },
                ],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: color_format,
                    blend: None,
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            cache: None,
            multiview_mask: None,
        });

        Self {
            pipeline,
            vertex_buffer,
            index_buffer,
            instance_buffer,
            uniform_buffer,
            bind_group,
            queue: queue.clone(),
        }
    }

    fn upload_positions(&self, positions: &[[f32; 3]]) {
        let n = positions.len().min(MAX_PARTICLES);
        if n == 0 {
            return;
        }
        let flat: Vec<f32> = positions[..n].iter().flat_map(|p| p.iter().copied()).collect();
        self.queue
            .write_buffer(&self.instance_buffer, 0, bytemuck::cast_slice(&flat));
    }

    fn draw(
        &self,
        pass: &mut RenderPass<'_>,
        view_projection: Mat4,
        point_scale: f32,
        instance_count: u32,
    ) {
        if instance_count == 0 {
            return;
        }
        let u = ParticleUniforms {
            view_projection: view_projection.to_cols_array(),
            point_scale,
        };
        self.queue
            .write_buffer(&self.uniform_buffer, 0, bytemuck::bytes_of(&u));
        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, &self.bind_group, &[]);
        pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        pass.set_vertex_buffer(1, self.instance_buffer.slice(..));
        pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
        pass.draw_indexed(0..6, 0, 0..instance_count);
    }
}

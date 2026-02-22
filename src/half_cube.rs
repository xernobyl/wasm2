//! Half-cube mesh: 3 visible faces, instanced. WebGPU-backed with same public API.

use wgpu::RenderPass;
use wgpu::util::DeviceExt;

const CUBE_WGSL: &str = include_str!("wgsl/cube.wgsl");
const CUBE_GBUFFER_WGSL: &str = include_str!("wgsl/cube_gbuffer.wgsl");

/// Half-cube: 3 faces meeting at corner (0,0,0), stored as a triangle fan (center 0 + 6 edge verts).
/// Positions in [0,1]^3. Indices: 6 triangles [0,1,2, 0,2,3, 0,3,4, 0,4,5, 0,5,6, 0,6,1].
/// Oriented each frame so the diagonal (1,1,1) points toward the camera (see rotation_diagonal_toward).
const INDICES: [u16; 18] = [
    0, 1, 2, 0, 2, 3, 0, 3, 4, 0, 4, 5, 0, 5, 6, 0, 6, 1,
];

/// Vertex: position only (3 f32). Normal derived in shader from position (x=1→+X, y=1→+Y, z=1→+Z; edge/corner = normalized diagonal).
#[rustfmt::skip]
const VERTICES: [f32; 21] = [
    0.0, 0.0, 0.0,
    1.0, 0.0, 0.0,
    1.0, 1.0, 0.0,
    0.0, 1.0, 0.0,
    0.0, 1.0, 1.0,
    0.0, 0.0, 1.0,
    1.0, 0.0, 1.0,
];

/// Max instances per draw (instance buffer size).
const MAX_INSTANCES: usize = 256;

/// Stub or WebGPU-backed half-cube.
#[derive(Debug)]
pub struct HalfCube {
    pub(crate) inner: Option<HalfCubeGpu>,
}

/// WebGPU pipeline and buffers for the half-cube.
#[derive(Debug)]
pub struct HalfCubeGpu {
    pipeline: wgpu::RenderPipeline,
    pipeline_gbuffer: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    index_count: u32,
    instance_buffer: wgpu::Buffer,
    view_projection_buffer: wgpu::Buffer,
    view_projection_gbuffer_buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
    bind_group_gbuffer: wgpu::BindGroup,
    queue: wgpu::Queue,
}

impl HalfCube {
    pub fn new() -> Self {
        Self { inner: None }
    }

    /// Called once when WebGPU is ready. Creates pipeline and buffers from the given device/queue.
    pub fn init_from_gpu(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        color_format: wgpu::TextureFormat,
    ) -> Self {
        Self {
            inner: Some(HalfCubeGpu::new(device, queue, color_format)),
        }
    }

    /// Upload instance model matrices (16 floats per matrix). No-op if WebGPU not in use.
    pub fn update_model(&mut self, matrices: &[f32]) {
        if let Some(ref mut g) = self.inner {
            g.upload_instances(matrices);
        }
    }

    /// Draw instanced. No-op if `pass` is None or WebGPU not in use.
    pub fn draw_instanced(
        &self,
        count: i32,
        pass: Option<&mut RenderPass<'_>>,
        view: &crate::view::ViewState,
    ) {
        if count <= 0 {
            return;
        }
        if let (Some(ref g), Some(p)) = (&self.inner, pass) {
            g.draw(p, view, count as u32);
        }
    }

    /// Draw instanced into a G-buffer pass (2 color + depth). Use when the current pass is the G-buffer pass.
    pub fn draw_instanced_gbuffer(
        &self,
        pass: &mut RenderPass<'_>,
        view: &crate::view::ViewState,
        count: i32,
    ) {
        if count <= 0 {
            return;
        }
        if let Some(ref g) = self.inner {
            g.draw_gbuffer(pass, view, count as u32);
        }
    }
}

impl HalfCubeGpu {
    fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        color_format: wgpu::TextureFormat,
    ) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("cube"),
            source: wgpu::ShaderSource::Wgsl(CUBE_WGSL.into()),
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("cube_bind_layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("cube_layout"),
            bind_group_layouts: &[&bind_group_layout],
            immediate_size: 0,
        });

        let view_projection_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("cube_view_projection"),
            size: 80,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("cube_vertices"),
            contents: bytemuck::cast_slice(&VERTICES),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("cube_indices"),
            contents: bytemuck::cast_slice(&INDICES),
            usage: wgpu::BufferUsages::INDEX,
        });

        let instance_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("cube_instances"),
            size: (MAX_INSTANCES * 16 * 4) as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("cube_bind_group"),
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: view_projection_buffer.as_entire_binding(),
            }],
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("cube"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs"),
                buffers: &[
                    wgpu::VertexBufferLayout {
                        array_stride: 12,
                        step_mode: wgpu::VertexStepMode::Vertex,
                        attributes: &[wgpu::VertexAttribute {
                            offset: 0,
                            shader_location: 0,
                            format: wgpu::VertexFormat::Float32x3,
                        }],
                    },
                    wgpu::VertexBufferLayout {
                        array_stride: 64,
                        step_mode: wgpu::VertexStepMode::Instance,
                        attributes: &[
                            wgpu::VertexAttribute {
                                offset: 0,
                                shader_location: 2,
                                format: wgpu::VertexFormat::Float32x4,
                            },
                            wgpu::VertexAttribute {
                                offset: 16,
                                shader_location: 3,
                                format: wgpu::VertexFormat::Float32x4,
                            },
                            wgpu::VertexAttribute {
                                offset: 32,
                                shader_location: 4,
                                format: wgpu::VertexFormat::Float32x4,
                            },
                            wgpu::VertexAttribute {
                                offset: 48,
                                shader_location: 5,
                                format: wgpu::VertexFormat::Float32x4,
                            },
                        ],
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
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Greater,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState::default(),
            cache: None,
            multiview_mask: None,
        });

        let shader_gbuffer = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("cube_gbuffer"),
            source: wgpu::ShaderSource::Wgsl(CUBE_GBUFFER_WGSL.into()),
        });
        let view_projection_gbuffer_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("cube_view_projection_gbuffer"),
            size: 256,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let bind_group_gbuffer = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("cube_bind_group_gbuffer"),
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: view_projection_gbuffer_buffer.as_entire_binding(),
            }],
        });
        let pipeline_gbuffer = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("cube_gbuffer"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader_gbuffer,
                entry_point: Some("vs"),
                buffers: &[
                    wgpu::VertexBufferLayout {
                        array_stride: 12,
                        step_mode: wgpu::VertexStepMode::Vertex,
                        attributes: &[wgpu::VertexAttribute { offset: 0, shader_location: 0, format: wgpu::VertexFormat::Float32x3 }],
                    },
                    wgpu::VertexBufferLayout {
                        array_stride: 64,
                        step_mode: wgpu::VertexStepMode::Instance,
                        attributes: &[
                            wgpu::VertexAttribute { offset: 0, shader_location: 2, format: wgpu::VertexFormat::Float32x4 },
                            wgpu::VertexAttribute { offset: 16, shader_location: 3, format: wgpu::VertexFormat::Float32x4 },
                            wgpu::VertexAttribute { offset: 32, shader_location: 4, format: wgpu::VertexFormat::Float32x4 },
                            wgpu::VertexAttribute { offset: 48, shader_location: 5, format: wgpu::VertexFormat::Float32x4 },
                        ],
                    },
                ],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader_gbuffer,
                entry_point: Some("fs"),
                targets: &[
                    Some(wgpu::ColorTargetState { format: wgpu::TextureFormat::Rgba16Float, blend: None, write_mask: wgpu::ColorWrites::ALL }),
                    Some(wgpu::ColorTargetState { format: wgpu::TextureFormat::Rg16Float, blend: None, write_mask: wgpu::ColorWrites::ALL }),
                ],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Greater,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState::default(),
            cache: None,
            multiview_mask: None,
        });

        Self {
            pipeline,
            pipeline_gbuffer,
            vertex_buffer,
            index_buffer,
            index_count: INDICES.len() as u32,
            instance_buffer,
            view_projection_buffer,
            view_projection_gbuffer_buffer,
            bind_group,
            bind_group_gbuffer,
            queue: queue.clone(),
        }
    }

    fn upload_instances(&self, matrices: &[f32]) {
        // matrices is num_instances * 16 floats
        let instance_count = matrices.len() / 16;
        let count = instance_count.min(MAX_INSTANCES);
        if count == 0 {
            return;
        }
        let bytes = bytemuck::cast_slice::<f32, u8>(&matrices[..count * 16]);
        self.queue.write_buffer(&self.instance_buffer, 0, bytes);
    }

    fn draw(&self, pass: &mut RenderPass<'_>, view: &crate::view::ViewState, count: u32) {
        let count = count.min(MAX_INSTANCES as u32);
        if count == 0 {
            return;
        }
        self.queue.write_buffer(
            &self.view_projection_buffer,
            0,
            bytemuck::bytes_of(&view.view_projection.to_cols_array()),
        );
        let cam_pos = view.inverse_view.col(3);
        let cam_pos_pad = [cam_pos.x, cam_pos.y, cam_pos.z, 0.0f32];
        self.queue.write_buffer(
            &self.view_projection_buffer,
            64,
            bytemuck::bytes_of(&cam_pos_pad),
        );
        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, &self.bind_group, &[]);
        pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        pass.set_vertex_buffer(1, self.instance_buffer.slice(..));
        pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
        pass.draw_indexed(0..self.index_count, 0, 0..count);
    }

    fn draw_gbuffer(&self, pass: &mut RenderPass<'_>, view: &crate::view::ViewState, count: u32) {
        let count = count.min(MAX_INSTANCES as u32);
        if count == 0 {
            return;
        }
        self.queue.write_buffer(
            &self.view_projection_gbuffer_buffer,
            0,
            bytemuck::bytes_of(&view.view_projection.to_cols_array()),
        );
        self.queue.write_buffer(
            &self.view_projection_gbuffer_buffer,
            64,
            bytemuck::bytes_of(&view.view_projection_no_jitter.to_cols_array()),
        );
        self.queue.write_buffer(
            &self.view_projection_gbuffer_buffer,
            128,
            bytemuck::bytes_of(&view.previous_view_projection_no_jitter.to_cols_array()),
        );
        let cam_pos = view.inverse_view.col(3);
        let cam_pos_pad = [cam_pos.x, cam_pos.y, cam_pos.z, 0.0f32];
        self.queue.write_buffer(
            &self.view_projection_gbuffer_buffer,
            192,
            bytemuck::bytes_of(&cam_pos_pad),
        );
        pass.set_pipeline(&self.pipeline_gbuffer);
        pass.set_bind_group(0, &self.bind_group_gbuffer, &[]);
        pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        pass.set_vertex_buffer(1, self.instance_buffer.slice(..));
        pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
        pass.draw_indexed(0..self.index_count, 0, 0..count);
    }
}

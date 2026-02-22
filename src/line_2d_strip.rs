//! 2D line strip for debug or UI. WebGPU-backed: vertex buffer (position xy + color rgb), pipeline.

use glam::Mat4;
use wgpu::RenderPass;

const LINE_2D_WGSL: &str = include_str!("wgsl/line_2d.wgsl");

/// Default max vertices for the line strip (resize buffer if needed).
const MAX_LINE_VERTICES: usize = 4096;

/// 2D line strip. Upload points (and optional colors) then draw with a view-projection matrix (e.g. ortho).
#[derive(Debug)]
pub struct Line2DStrip {
    vertices: Vec<[f32; 5]>, // xy, rgb
    gpu: Option<Line2DStripGpu>,
}

/// WebGPU pipeline and buffers for the line strip.
#[derive(Debug)]
pub struct Line2DStripGpu {
    pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    view_projection_buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
    queue: wgpu::Queue,
    vertex_count: u32,
}

impl Line2DStrip {
    pub fn new() -> Self {
        Self {
            vertices: Vec::new(),
            gpu: None,
        }
    }

    /// Set line points (xy). Colors default to white. Call after [`upload_to_gpu`](Self::upload_to_gpu) to update the buffer.
    pub fn set_points(&mut self, points: &[[f32; 2]]) {
        self.vertices.clear();
        for p in points {
            self.vertices.push([p[0], p[1], 1.0, 1.0, 1.0]);
        }
    }

    /// Set line points with per-vertex color (xy, rgb). Lengths must match.
    pub fn set_points_with_colors(&mut self, points: &[[f32; 2]], colors: &[[f32; 3]]) {
        self.vertices.clear();
        let n = points.len().min(colors.len());
        for i in 0..n {
            self.vertices.push([
                points[i][0],
                points[i][1],
                colors[i][0],
                colors[i][1],
                colors[i][2],
            ]);
        }
    }

    /// Upload to GPU and create pipeline. Call when WebGPU is ready.
    pub fn upload_to_gpu(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        color_format: wgpu::TextureFormat,
    ) {
        self.gpu = Some(Line2DStripGpu::new(
            device,
            queue,
            color_format,
            MAX_LINE_VERTICES,
        ));
    }

    /// Update GPU vertex buffer from current [`vertices`](Self::set_points). No-op if not uploaded or vertex count is 0.
    pub fn update_buffer(&self) {
        if self.vertices.is_empty() {
            return;
        }
        if let Some(ref g) = self.gpu {
            g.upload_vertices(&self.vertices);
        }
    }

    /// Draw the line strip. Call [`update_buffer`](Self::update_buffer) first if points changed. view_projection is typically an ortho matrix for 2D.
    pub fn draw(
        &self,
        pass: Option<&mut RenderPass<'_>>,
        view_projection: Mat4,
    ) {
        if self.vertices.is_empty() {
            return;
        }
        if let (Some(ref g), Some(p)) = (&self.gpu, pass) {
            g.draw(p, view_projection, self.vertices.len() as u32);
        }
    }
}

impl Default for Line2DStrip {
    fn default() -> Self {
        Self::new()
    }
}

impl Line2DStripGpu {
    fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        color_format: wgpu::TextureFormat,
        max_vertices: usize,
    ) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("line_2d"),
            source: wgpu::ShaderSource::Wgsl(LINE_2D_WGSL.into()),
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("line_2d_bind_layout"),
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
            label: Some("line_2d_layout"),
            bind_group_layouts: &[&bind_group_layout],
            immediate_size: 0,
        });

        let view_projection_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("line_2d_view_projection"),
            size: 64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("line_2d_vertices"),
            size: (max_vertices * 5 * 4) as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("line_2d_bind_group"),
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: view_projection_buffer.as_entire_binding(),
            }],
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("line_2d"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs"),
                buffers: &[wgpu::VertexBufferLayout {
                    array_stride: 20,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &[
                        wgpu::VertexAttribute {
                            offset: 0,
                            shader_location: 0,
                            format: wgpu::VertexFormat::Float32x2,
                        },
                        wgpu::VertexAttribute {
                            offset: 8,
                            shader_location: 1,
                            format: wgpu::VertexFormat::Float32x3,
                        },
                    ],
                }],
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
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::LineStrip,
                ..Default::default()
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            cache: None,
            multiview_mask: None,
        });

        Self {
            pipeline,
            vertex_buffer,
            view_projection_buffer,
            bind_group,
            queue: queue.clone(),
            vertex_count: 0,
        }
    }

    fn upload_vertices(&self, vertices: &[[f32; 5]]) {
        let n = vertices.len().min(MAX_LINE_VERTICES);
        if n == 0 {
            return;
        }
        let flat: Vec<f32> = vertices[..n].iter().flat_map(|v| v.iter().copied()).collect();
        self.queue
            .write_buffer(&self.vertex_buffer, 0, bytemuck::cast_slice(&flat));
    }

    fn draw(&self, pass: &mut RenderPass<'_>, view_projection: Mat4, vertex_count: u32) {
        if vertex_count == 0 {
            return;
        }
        let vp = view_projection.to_cols_array();
        self.queue
            .write_buffer(&self.view_projection_buffer, 0, bytemuck::cast_slice(&vp));
        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, &self.bind_group, &[]);
        pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        pass.draw(0..vertex_count, 0..1);
    }
}

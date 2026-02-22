//! Minecraft-style voxel chunk with greedy meshing (culling + quad merging).
//! Only visible faces are drawn; adjacent same-direction faces are merged into quads.
//! ChunkMesh holds mesh data and optional WebGPU buffers/pipeline for drawing.

/// 3D voxel grid. 0 = air, non-zero = solid.
pub struct Chunk {
    pub nx: usize,
    pub ny: usize,
    pub nz: usize,
    data: Vec<u8>,
}

impl Chunk {
    pub fn new(nx: usize, ny: usize, nz: usize) -> Self {
        Chunk {
            nx,
            ny,
            nz,
            data: vec![0; nx * ny * nz],
        }
    }

    #[inline]
    fn index(&self, x: usize, y: usize, z: usize) -> usize {
        x + y * self.nx + z * self.nx * self.ny
    }

    #[inline]
    pub fn get(&self, x: usize, y: usize, z: usize) -> u8 {
        self.data[self.index(x, y, z)]
    }

    #[inline]
    pub fn set(&mut self, x: usize, y: usize, z: usize, v: u8) {
        let i = self.index(x, y, z);
        self.data[i] = v;
    }

    #[inline]
    fn solid(&self, x: usize, y: usize, z: usize) -> bool {
        self.get(x, y, z) != 0
    }

    /// Fill with a hollow box (walls only) for testing.
    pub fn fill_hollow_box(&mut self) {
        for x in 0..self.nx {
            for y in 0..self.ny {
                for z in 0..self.nz {
                    let on_boundary = x == 0 || x == self.nx - 1
                        || y == 0 || y == self.ny - 1
                        || z == 0 || z == self.nz - 1;
                    if on_boundary {
                        self.set(x, y, z, 1);
                    }
                }
            }
        }
    }

    /// Fill with a solid sphere (center at chunk center, radius ~min/2).
    /// Kept for API / future use; do not remove.
    #[allow(dead_code)]
    pub fn fill_sphere(&mut self) {
        let cx = (self.nx as f32 - 1.0) * 0.5;
        let cy = (self.ny as f32 - 1.0) * 0.5;
        let cz = (self.nz as f32 - 1.0) * 0.5;
        let r = (cx.min(cy).min(cz)) * 0.9;
        for x in 0..self.nx {
            for y in 0..self.ny {
                for z in 0..self.nz {
                    let dx = x as f32 - cx;
                    let dy = y as f32 - cy;
                    let dz = z as f32 - cz;
                    if dx * dx + dy * dy + dz * dz <= r * r {
                        self.set(x, y, z, 1);
                    }
                }
            }
        }
    }

    /// Greedy mesh: build vertex + index buffers (position.xyz, normal.xyz per vertex).
    /// Origin is chunk corner (0,0,0); caller applies model matrix for world position.
    pub fn build_greedy_mesh(&self) -> (Vec<f32>, Vec<u32>) {
        let mut vertices: Vec<f32> = Vec::new();
        let mut indices: Vec<u32> = Vec::new();
        let mut base_index = 0u32;

        // Face direction: (axis 0=x,1=y,2=z), sign ±1, normal vector
        let directions: [(usize, i32, [f32; 3]); 6] = [
            (0, 1, [1.0, 0.0, 0.0]),   // +X
            (0, -1, [-1.0, 0.0, 0.0]),  // -X
            (1, 1, [0.0, 1.0, 0.0]),   // +Y
            (1, -1, [0.0, -1.0, 0.0]),  // -Y
            (2, 1, [0.0, 0.0, 1.0]),   // +Z
            (2, -1, [0.0, 0.0, -1.0]),  // -Z
        ];

        for (axis, sign, normal) in directions {
            let (nx, ny, nz) = (self.nx as i32, self.ny as i32, self.nz as i32);
            // Layer size in the two other dimensions
            let (da, db, dc) = match axis {
                0 => (ny, nz, nx),
                1 => (nx, nz, ny),
                _ => (nx, ny, nz),
            };
            for c in 0..dc {
                let c_usize = c as usize;
                // 2D grid of "face visible" in (a, b)
                let mut layer = vec![false; (da as usize) * (db as usize)];
                for a in 0..da {
                    for b in 0..db {
                        let (x, y, z) = match axis {
                            0 => (c_usize, a as usize, b as usize),
                            1 => (a as usize, c_usize, b as usize),
                            _ => (a as usize, b as usize, c_usize),
                        };
                        let solid_here = x < self.nx && y < self.ny && z < self.nz && self.solid(x, y, z);
                        let (nx2, ny2, nz2) = match axis {
                            0 => (x + (sign > 0) as usize, y, z),
                            1 => (x, y + (sign > 0) as usize, z),
                            _ => (x, y, z + (sign > 0) as usize),
                        };
                        let neighbor_empty = match axis {
                            0 => (sign > 0 && nx2 < self.nx && !self.solid(nx2, y, z))
                                || (sign < 0 && c_usize > 0 && !self.solid(c_usize - 1, y, z)),
                            1 => (sign > 0 && ny2 < self.ny && !self.solid(x, ny2, z))
                                || (sign < 0 && c_usize > 0 && !self.solid(x, c_usize - 1, z)),
                            _ => (sign > 0 && nz2 < self.nz && !self.solid(x, y, nz2))
                                || (sign < 0 && c_usize > 0 && !self.solid(x, y, c_usize - 1)),
                        };
                        if solid_here && neighbor_empty {
                            layer[(a as usize) + (b as usize) * (da as usize)] = true;
                        }
                    }
                }
                // Greedy merge: find axis-aligned rectangles (expand using layer, mark in used)
                let mut used = vec![false; layer.len()];
                let da_usize = da as usize;
                let _db_usize = db as usize;
                for b in 0..db {
                    let b_usize = b as usize;
                    for a in 0..da {
                        let a_usize = a as usize;
                        let idx = a_usize + b_usize * da_usize;
                        if !layer[idx] || used[idx] {
                            continue;
                        }
                        let mut w = 0i32;
                        while (a + w) < da
                            && layer[(a_usize + w as usize) + b_usize * da_usize]
                        {
                            w += 1;
                        }
                        let mut h = 0i32;
                        'h: while (b + h) < db {
                            for aw in 0..w {
                                if !layer[(a_usize + aw as usize)
                                    + (b_usize + h as usize) * da_usize]
                                {
                                    break 'h;
                                }
                            }
                            h += 1;
                        }
                        for hh in 0..h {
                            for ww in 0..w {
                                used[(a_usize + ww as usize)
                                    + (b_usize + hh as usize) * da_usize] = true;
                            }
                        }
                        // Emit quad (a, b) to (a+w, b+h) in layer space
                        let (x0, y0, z0) = match axis {
                            0 => (
                                c as f32 + (sign > 0) as i32 as f32,
                                a as f32,
                                b as f32,
                            ),
                            1 => (
                                a as f32,
                                c as f32 + (sign > 0) as i32 as f32,
                                b as f32,
                            ),
                            _ => (
                                a as f32,
                                b as f32,
                                c as f32 + (sign > 0) as i32 as f32,
                            ),
                        };
                        let (x1, y1, z1) = match axis {
                            0 => (
                                c as f32 + (sign > 0) as i32 as f32,
                                (a + w) as f32,
                                (b + h) as f32,
                            ),
                            1 => (
                                (a + w) as f32,
                                c as f32 + (sign > 0) as i32 as f32,
                                (b + h) as f32,
                            ),
                            _ => (
                                (a + w) as f32,
                                (b + h) as f32,
                                c as f32 + (sign > 0) as i32 as f32,
                            ),
                        };
                        // Four corners of the quad (order for CCW front face)
                        let (v0, v1, v2, v3) = match axis {
                            0 => {
                                let x = x0;
                                (
                                    [x, y0, z0],
                                    [x, y1, z0],
                                    [x, y1, z1],
                                    [x, y0, z1],
                                )
                            }
                            1 => {
                                let y = y0;
                                (
                                    [x0, y, z0],
                                    [x0, y, z1],
                                    [x1, y, z1],
                                    [x1, y, z0],
                                )
                            }
                            _ => {
                                let z = z0;
                                (
                                    [x0, y0, z],
                                    [x0, y1, z],
                                    [x1, y1, z],
                                    [x1, y0, z],
                                )
                            }
                        };
                        for v in [v0, v1, v2, v3] {
                            vertices.extend_from_slice(&v);
                            vertices.extend_from_slice(&normal);
                        }
                        indices.extend_from_slice(&[
                            base_index,
                            base_index + 1,
                            base_index + 2,
                            base_index,
                            base_index + 2,
                            base_index + 3,
                        ]);
                        base_index += 4;
                    }
                }
            }
        }

        (vertices, indices)
    }
}

const CHUNK_WGSL: &str = include_str!("wgsl/chunk.wgsl");

/// Mesh data for a chunk (greedy-meshed quads). Optionally has WebGPU buffers for drawing.
pub struct ChunkMesh {
    /// Vertex data (position.xyz, normal.xyz per vertex); 6 floats per vertex.
    pub vertices: Vec<f32>,
    /// Triangle indices.
    pub indices: Vec<u32>,
    pub(crate) gpu: Option<ChunkMeshGpu>,
}

/// WebGPU pipeline and buffers for a chunk mesh (single draw, no instancing).
pub struct ChunkMeshGpu {
    pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    index_count: u32,
    view_projection_buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
    queue: wgpu::Queue,
}

impl ChunkMesh {
    /// Build mesh from chunk. No GPU upload until [`upload_to_gpu`](Self::upload_to_gpu).
    pub fn from_chunk(chunk: &Chunk) -> Self {
        let (vertices, indices) = chunk.build_greedy_mesh();
        Self {
            vertices,
            indices,
            gpu: None,
        }
    }

    /// Upload mesh to GPU and create pipeline. Call when WebGPU is ready. Idempotent: re-upload replaces buffers.
    pub fn upload_to_gpu(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        color_format: wgpu::TextureFormat,
    ) {
        if self.vertices.is_empty() || self.indices.is_empty() {
            return;
        }
        self.gpu = Some(ChunkMeshGpu::new(
            device,
            queue,
            color_format,
            &self.vertices,
            &self.indices,
        ));
    }

    /// Draw the chunk mesh. No-op if [`upload_to_gpu`](Self::upload_to_gpu) was not called or pass is None.
    pub fn draw(
        &self,
        pass: Option<&mut wgpu::RenderPass<'_>>,
        view: &crate::view::ViewState,
    ) {
        if let (Some(ref g), Some(p)) = (&self.gpu, pass) {
            g.draw(p, view);
        }
    }
}

impl ChunkMeshGpu {
    fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        color_format: wgpu::TextureFormat,
        vertices: &[f32],
        indices: &[u32],
    ) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("chunk"),
            source: wgpu::ShaderSource::Wgsl(CHUNK_WGSL.into()),
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("chunk_bind_layout"),
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
            label: Some("chunk_layout"),
            bind_group_layouts: &[&bind_group_layout],
            immediate_size: 0,
        });

        let view_projection_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("chunk_view_projection"),
            size: 64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("chunk_vertices"),
            size: (vertices.len() * 4) as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        queue.write_buffer(&vertex_buffer, 0, bytemuck::cast_slice(vertices));

        let index_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("chunk_indices"),
            size: (indices.len() * 4) as u64,
            usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        queue.write_buffer(&index_buffer, 0, bytemuck::cast_slice(indices));

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("chunk_bind_group"),
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: view_projection_buffer.as_entire_binding(),
            }],
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("chunk"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs"),
                buffers: &[wgpu::VertexBufferLayout {
                    array_stride: 24,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &[
                        wgpu::VertexAttribute {
                            offset: 0,
                            shader_location: 0,
                            format: wgpu::VertexFormat::Float32x3,
                        },
                        wgpu::VertexAttribute {
                            offset: 12,
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
            index_count: indices.len() as u32,
            view_projection_buffer,
            bind_group,
            queue: queue.clone(),
        }
    }

    fn draw(&self, pass: &mut wgpu::RenderPass<'_>, view: &crate::view::ViewState) {
        let vp = view.view_projection_no_jitter.to_cols_array();
        self.queue
            .write_buffer(&self.view_projection_buffer, 0, bytemuck::cast_slice(&vp));
        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, &self.bind_group, &[]);
        pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
        pass.draw_indexed(0..self.index_count, 0, 0..1);
    }
}

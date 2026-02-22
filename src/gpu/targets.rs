//! Render targets: G-buffer (color, depth reversed-Z, velocity), TAA history, resolve.

/// One set of G-buffer + TAA + blur for a single viewport.
#[derive(Debug)]
pub struct GbufferSet {
    pub color: wgpu::Texture,
    pub depth: wgpu::Texture,
    pub velocity: wgpu::Texture,
    pub resolve: wgpu::Texture,
    /// Ping-pong: [0] and [1] for TAA history.
    pub history: [wgpu::Texture; 2],
    /// Half-res for bloom brightness extraction.
    pub bloom: wgpu::Texture,
    /// Half-res Kawase blur output.
    pub blur: wgpu::Texture,
    pub width: u32,
    pub height: u32,
}

impl GbufferSet {
    pub fn color_view(&self) -> wgpu::TextureView {
        self.color.create_view(&Default::default())
    }
    pub fn depth_view(&self) -> wgpu::TextureView {
        self.depth.create_view(&Default::default())
    }
    pub fn velocity_view(&self) -> wgpu::TextureView {
        self.velocity.create_view(&Default::default())
    }
    pub fn resolve_view(&self) -> wgpu::TextureView {
        self.resolve.create_view(&Default::default())
    }
    pub fn history_view(&self, index: usize) -> wgpu::TextureView {
        self.history[index].create_view(&Default::default())
    }
    pub fn bloom_view(&self) -> wgpu::TextureView {
        self.bloom.create_view(&Default::default())
    }
    pub fn blur_view(&self) -> wgpu::TextureView {
        self.blur.create_view(&Default::default())
    }

    pub fn bloom_width(&self) -> u32 {
        (self.width / 2).max(1)
    }
    pub fn bloom_height(&self) -> u32 {
        (self.height / 2).max(1)
    }

    /// Create or recreate targets for the given size. Uses reversed-Z depth (GREATER, clear 0).
    pub fn new(device: &wgpu::Device, width: u32, height: u32) -> Self {
        let color = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("gbuffer_color"),
            size: wgpu::Extent3d { width, height, depth_or_array_layers: 1 },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba16Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        let depth = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("gbuffer_depth"),
            size: wgpu::Extent3d { width, height, depth_or_array_layers: 1 },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        let velocity = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("gbuffer_velocity"),
            size: wgpu::Extent3d { width, height, depth_or_array_layers: 1 },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rg16Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        let resolve = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("taa_resolve"),
            size: wgpu::Extent3d { width, height, depth_or_array_layers: 1 },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba16Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        let history = [
            device.create_texture(&wgpu::TextureDescriptor {
                label: Some("taa_history_0"),
                size: wgpu::Extent3d { width, height, depth_or_array_layers: 1 },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba16Float,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
                view_formats: &[],
            }),
            device.create_texture(&wgpu::TextureDescriptor {
                label: Some("taa_history_1"),
                size: wgpu::Extent3d { width, height, depth_or_array_layers: 1 },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba16Float,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
                view_formats: &[],
            }),
        ];
        let bw = (width / 2).max(1);
        let bh = (height / 2).max(1);
        let bloom = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("bloom"),
            size: wgpu::Extent3d { width: bw, height: bh, depth_or_array_layers: 1 },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba16Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        let blur = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("blur"),
            size: wgpu::Extent3d { width: bw, height: bh, depth_or_array_layers: 1 },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba16Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        Self {
            color,
            depth,
            velocity,
            resolve,
            history,
            bloom,
            blur,
            width,
            height,
        }
    }
}

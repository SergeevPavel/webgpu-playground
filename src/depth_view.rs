use wgpu::{ColorTargetState, CommandEncoder, Device, Face, FragmentState, StoreOp, SurfaceConfiguration, TextureFormat, TextureView, VertexState};

pub struct DepthView {
    pipeline: wgpu::RenderPipeline,
}

impl DepthView {
    pub(crate) fn new(device: &Device, target_texture_format: TextureFormat) -> DepthView {
        let pipeline = Self::create_depth_render_pipeline(device, target_texture_format);
        DepthView { pipeline }
    }

    pub fn create_depth_render_pipeline(device: &Device,
                                        target_texture_format: TextureFormat) -> wgpu::RenderPipeline {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Depth view shaders"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shaders/depth_render.wgsl").into()),
        });
        return device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Depth View Pipeline"),
            layout: None,
            vertex: VertexState {
                module: &shader,
                entry_point: "depth_view_vs",
                compilation_options: Default::default(),
                buffers: &[],
            },
            fragment: Some(FragmentState {
                module: &shader,
                entry_point: "depth_view_fs",
                compilation_options: Default::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format: target_texture_format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                unclipped_depth: false,
                polygon_mode: wgpu::PolygonMode::Fill,
                conservative: false,
            },
            depth_stencil: None,
            multisample: Default::default(),
            multiview: None,
            cache: None,
        })
    }

    pub fn render<'a>(&self, view: &TextureView, encoder: &mut CommandEncoder) {
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Depth View Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        render_pass.set_pipeline(&self.pipeline);
        render_pass.draw(0..3, 0..1);
    }
}
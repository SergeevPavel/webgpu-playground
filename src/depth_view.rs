use wgpu::{BindGroup, BindGroupLayout, ColorTargetState, CommandEncoder, Device, Face, FragmentState, StoreOp, SurfaceConfiguration, TextureFormat, TextureView, VertexState};
use wgpu::TextureSampleType::Depth;
use crate::texture::Texture;

pub struct DepthView {
    pipeline: wgpu::RenderPipeline,
    depth_texture_bind_group_layout: wgpu::BindGroupLayout,
    depth_texture_bind_group: wgpu::BindGroup,
}

impl DepthView {
    pub(crate) fn new(device: &Device,
                      target_texture_format: TextureFormat,
                      depth_texture: &Texture) -> DepthView {
        let depth_texture_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("depth_texture_bind_group_layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: Depth,
                        view_dimension: Default::default(),
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Comparison),
                    count: None,
                }
            ]
        });
        let pipeline = Self::create_depth_render_pipeline(device, target_texture_format, &[&depth_texture_bind_group_layout]);
        let depth_texture_bind_group = Self::create_bind_group(device, &depth_texture_bind_group_layout, depth_texture);
        DepthView { pipeline, depth_texture_bind_group_layout, depth_texture_bind_group }
    }

    fn create_bind_group(device: &Device,
                         depth_texture_bind_group_layout: &BindGroupLayout,
                         depth_texture: &Texture) -> BindGroup {
        return device.create_bind_group(
            &wgpu::BindGroupDescriptor {
                label: Some("depth_texture_bind_group"),
                layout: depth_texture_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&depth_texture.view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&depth_texture.sampler),
                    }
                ],
            }
        );
    }

    pub fn set_depth_texture(&mut self, device: &Device, depth_texture: &Texture) {
        self.depth_texture_bind_group = Self::create_bind_group(device, &self.depth_texture_bind_group_layout, depth_texture);
    }

    pub fn create_depth_render_pipeline(device: &Device,
                                        target_texture_format: TextureFormat,
                                        bind_group_layouts: &[&BindGroupLayout]) -> wgpu::RenderPipeline {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Depth view shaders"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shaders/depth_render.wgsl").into()),
        });
        let depth_view_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Depth View Pipeline Layout"),
                bind_group_layouts,
                push_constant_ranges: &[],
            });
        device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Depth View Pipeline"),
            layout: Some(&depth_view_pipeline_layout),
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

    pub fn render(&self, view: &TextureView, encoder: &mut CommandEncoder) {
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
        render_pass.set_bind_group(0, &self.depth_texture_bind_group, &[]);
        render_pass.draw(0..3, 0..1);
    }
}
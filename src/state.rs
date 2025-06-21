use std::f64::consts::PI;

use wgpu::{BindGroupLayout, CommandEncoder, Device, StoreOp, SurfaceConfiguration, TextureView};
use wgpu::hal::empty::Encoder;
use winit::{
    dpi::PhysicalPosition,
    event::WindowEvent,
    window::Window,
};

use crate::instances::{Instances, Rotation};
use crate::mesh::{Mesh, Vertex};
use crate::{camera::{CameraState}, texture::{self, Texture}};
use crate::depth_view::DepthView;

pub struct State<'a> {
    surface: wgpu::Surface<'a>,
    window: &'a Window,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    pub size: winit::dpi::PhysicalSize<u32>,
    background_color: wgpu::Color,
    render_pipeline: wgpu::RenderPipeline,
    mesh: Mesh,
    texture_bind_group: wgpu::BindGroup,
    camera_state: CameraState,
    rotator: Rotation,
    pub instances: Instances,
    depth_texture: Texture,
    depth_view: Option<DepthView>
}

impl <'a> State<'a> {
    // Creating some of the wgpu types requires async code
    pub async fn new(window: &'a Window) -> Self {
        let size = window.inner_size();

        // The instance is a handle to our GPU
        // Backends::all => Vulkan + Metal + DX12 + Browser WebGPU
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            flags: Default::default(),
            dx12_shader_compiler: Default::default(),
            gles_minor_version: Default::default(),
        });

        let surface = instance.create_surface(window).unwrap();

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    // WebGL doesn't support all of wgpu's features, so if
                    // we're building for the web we'll have to disable some.
                    required_limits: if cfg!(target_arch = "wasm32") {
                        wgpu::Limits::downlevel_webgl2_defaults()
                    } else {
                        wgpu::Limits::default()
                    },
                    label: None,
                    required_features: Default::default(),
                    memory_hints: Default::default(),
                },
                None, // Trace path
            )
            .await
            .unwrap();

        let surface_caps = surface.get_capabilities(&adapter);
        // Shader code in this tutorial assumes an sRGB surface texture. Using a different
        // one will result all the colors coming out darker. If you want to support non
        // sRGB surfaces, you'll need to account for that when drawing to the frame.
        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(surface_caps.formats[0]);
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: surface_caps.present_modes[0],
            desired_maximum_frame_latency: 1,
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
        };
        surface.configure(&device, &config);

        let tree_texture_bytes = include_bytes!("textures/happy-tree.png");
        let tree_texture = texture::Texture::from_bytes(&device, &queue, tree_texture_bytes, "happy-tree.png").unwrap();

        let texture_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        // This should match the filterable field of the
                        // corresponding Texture entry above.
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
                label: Some("texture_bind_group_layout"),
            });

        let texture_bind_group = device.create_bind_group(
            &wgpu::BindGroupDescriptor {
                layout: &texture_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&tree_texture.view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&tree_texture.sampler),
                    }
                ],
                label: Some("diffuse_bind_group"),
            }
        );

        let depth_texture = Texture::create_depth_texture(&device, &config, "depth_texture");

        let mesh = Mesh::new(&device);

        let camera_bind_group_layout = CameraState::layout(&device);
        let camera_state = CameraState::new(&device, config.width, config.height, &camera_bind_group_layout);

        let rotator_bind_group_layout = Rotation::layout(&device);
        let rotator = Rotation::new(&device, &rotator_bind_group_layout);
        let instances = Instances::new(&device);

        let bind_group_layouts = [
            &texture_bind_group_layout,
            &camera_bind_group_layout,
            &rotator_bind_group_layout,
            &instances.layout
        ];
        let render_pipeline = Self::create_render_scene_pipeline(&device, &config, &bind_group_layouts);
        let depth_view = DepthView::new(&device, config.format, &depth_texture);

        Self {
            surface,
            window,
            device,
            queue,
            config,
            size,
            background_color: position_to_color(&PhysicalPosition { x: 0f64, y: 0f64 }),
            render_pipeline,
            mesh,
            camera_state,
            rotator,
            instances,
            texture_bind_group,
            depth_texture,
            depth_view: Some(depth_view)
        }
    }

    pub fn create_render_scene_pipeline(
        device: &Device,
        config: &SurfaceConfiguration,
        bind_group_layouts: &[&BindGroupLayout]
    ) -> wgpu::RenderPipeline {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Just some shaders"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shaders/shaders.wgsl").into()),
        });
        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts,
                push_constant_ranges: &[],
            });

        return device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                compilation_options: Default::default(),
                buffers: &[Vertex::desc()],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                compilation_options: Default::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                // Setting this to anything other than Fill requires Features::NON_FILL_POLYGON_MODE
                polygon_mode: wgpu::PolygonMode::Fill,
                // Requires Features::DEPTH_CLIP_CONTROL
                unclipped_depth: false,
                // Requires Features::CONSERVATIVE_RASTERIZATION
                conservative: false,
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: Texture::DEPTH_FORMAT,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
            cache: None,
        });
    }

    pub fn window(&self) -> &Window {
        &self.window
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 && new_size.width <= 8192 && new_size.height <= 8192 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
            self.depth_texture = Texture::create_depth_texture(&self.device, &self.config, "depth_texture");
            match &mut self.depth_view {
                Some(depth_view) => {
                    depth_view.set_depth_texture(&self.device, &self.depth_texture);
                }
                _ => {}
            }
        }
    }

    pub fn input(&mut self, event: &WindowEvent) -> bool {
        match event {
            WindowEvent::CursorMoved { position, .. } => {
                self.background_color = position_to_color(position);
                true
            }
            _ => {
                self.camera_state.controller.process_events(event)
            },
        }
    }

    pub fn update(&mut self) {
        self.camera_state.update(&self.queue);
        self.rotator.update(&self.queue);
    }

    fn run_cubes_pipeline(&self, view: &TextureView, encoder: &mut CommandEncoder) {
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(self.background_color),
                    store: StoreOp::Store,
                },
            })],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: &self.depth_texture.view,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(1.0),
                    store: StoreOp::Store,
                }),
                stencil_ops: None,
            }),
            timestamp_writes: None,
            occlusion_query_set: None,
        });
        render_pass.set_pipeline(&self.render_pipeline);
        render_pass.set_bind_group(0, &self.texture_bind_group, &[]);
        render_pass.set_bind_group(1, &self.camera_state.bind_group, &[]);
        render_pass.set_bind_group(2, &self.rotator.bind_group, &[]);
        render_pass.set_bind_group(3, &self.instances.bind_group, &[]);
        render_pass.set_vertex_buffer(0, self.mesh.vertex_buffer.slice(..));
        render_pass.set_index_buffer(self.mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
        render_pass.draw_indexed(0..self.mesh.num_indices, 0, 0..self.instances.count());

    }

    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });
        self.run_cubes_pipeline(&view, &mut encoder);
        if let Some(depth_view) = &self.depth_view {
            depth_view.render(&view, &mut encoder);
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}

fn position_to_color(p: &PhysicalPosition<f64>) -> wgpu::Color {
    wgpu::Color {
        r: ((p.x * PI / 128.0).cos() + 1.0) / 2.0,
        g: ((p.y * PI / 128.0).sin() + 1.0) / 2.0,
        b: (((p.x + p.y) * PI / 256.0).cos() + 1.0) / 2.0,
        a: 1.0,
    }
}
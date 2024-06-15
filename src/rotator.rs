use cgmath::{Deg, prelude::*};
use wgpu::BindGroupLayout;
use wgpu::util::DeviceExt;

pub struct Rotator {
    pub step: cgmath::Matrix4<f32>,
    pub rotation: cgmath::Matrix4<f32>,
    pub rotation_uniform: RotatorUniform,
    pub buffer: wgpu::Buffer,
    pub bind_group: wgpu::BindGroup,
}

impl Rotator {
    pub fn new(device: &wgpu::Device, layout: &BindGroupLayout) -> Self {
        let x_step = cgmath::Matrix4::from_angle_x(Deg(1f32));
        let y_step = cgmath::Matrix4::from_angle_y(Deg(0.8f32));
        let step = x_step * y_step;
        let rotation = cgmath::Matrix4::identity();
        let rotation_uniform = RotatorUniform {
            rotation: rotation.into(),
        };

        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Rotator Buffer"),
            contents: bytemuck::cast_slice(&[rotation_uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: buffer.as_entire_binding(),
                }
            ],
            label: Some("rotator_bind_group"),
        });

        Self {
            step,
            rotation,
            rotation_uniform,
            buffer,
            bind_group
        }
    }

    pub fn layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("rotator_bind_group_layout"),
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
        })
    }

    pub fn update(&mut self, queue: &wgpu::Queue) {
        self.rotation = self.rotation * self.step;
        let rotation_uniform = RotatorUniform {
            rotation: self.rotation.into(),
        };
        self.rotation_uniform = rotation_uniform;
        queue.write_buffer(&self.buffer, 0, bytemuck::cast_slice(&[self.rotation_uniform]))
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct RotatorUniform {
    rotation: [[f32; 4]; 4],
}
use std::num::NonZeroU32;
use cgmath::{Deg, Matrix4, prelude::*, Vector3};
use wgpu::{BindGroupLayout, Buffer};
use wgpu::util::DeviceExt;

pub struct Rotation {
    pub step: cgmath::Matrix4<f32>,
    pub rotation: cgmath::Matrix4<f32>,
    pub rotation_uniform: PodMatrix,
    pub buffer: wgpu::Buffer,
    pub bind_group: wgpu::BindGroup,
}

impl Rotation {
    pub fn new(device: &wgpu::Device, layout: &BindGroupLayout) -> Self {
        let x_step = cgmath::Matrix4::from_angle_x(Deg(1f32));
        let y_step = cgmath::Matrix4::from_angle_y(Deg(0.8f32));
        let step = x_step * y_step;
        let rotation = cgmath::Matrix4::identity();
        let rotation_uniform = PodMatrix {
            m: rotation.into(),
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
        let rotation_uniform: PodMatrix = self.rotation.into();
        self.rotation_uniform = rotation_uniform;
        queue.write_buffer(&self.buffer, 0, bytemuck::cast_slice(&[self.rotation_uniform]))
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct PodMatrix {
    m: [[f32; 4]; 4],
}

impl From<Matrix4<f32>> for PodMatrix {
    fn from(value: Matrix4<f32>) -> Self {
        return PodMatrix {
            m: value.into(),
        }
    }
}

pub struct Instances {
    pub step: cgmath::Matrix4<f32>,
    pub transformations: Vec<cgmath::Matrix4<f32>>,
    pub layout: wgpu::BindGroupLayout,
    pub buffer: wgpu::Buffer,
    pub bind_group: wgpu::BindGroup,
}

impl Instances {
    fn layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("instances_bind_group_layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: true },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        })
    }

    fn step() -> Matrix4<f32> {
        let x_step = cgmath::Matrix4::from_angle_x(Deg(1f32));
        let y_step = cgmath::Matrix4::from_angle_y(Deg(0.8f32));
        let step = x_step * y_step;
        return step;
    }

    pub fn count(&self) -> u32 {
        return self.transformations.len() as u32;
    }

    pub fn new(device: &wgpu::Device) -> Self {
        let per_row = 4i32;
        let per_col = 4i32;
        let count = (per_col * per_row) as usize;
        let dx = 2.0f32;
        let dy = 2.0f32;
        let mut transformations = Vec::with_capacity(count);
        for i in 0..=per_row {
            for j in 0..per_col {
                let x = (j - per_row / 2) as f32 * dx;
                let y = (i - per_col / 2) as f32 * dy;
                let m = Matrix4::from_translation(Vector3::new(x, y, 0f32));
                transformations.push(m);
            }
        }
        let layout = Self::layout(device);
        let pod_transformations: Vec<PodMatrix> = transformations.iter().map(|t| {
            (*t).into()
        }).collect();

        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Instances Buffer"),
            contents: bytemuck::cast_slice(pod_transformations.as_slice()),
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: buffer.as_entire_binding(),
                }
            ],
            label: Some("instances_bind_group"),
        });

        Self {
            step: Self::step(),
            transformations,
            layout,
            buffer,
            bind_group
        }
    }
}
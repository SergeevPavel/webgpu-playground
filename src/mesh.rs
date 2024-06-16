use wgpu::Device;
use wgpu::util::DeviceExt;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    position: [f32; 3],
    tex_coords: [f32; 2],
}

impl Vertex {
    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x2,
                },
            ],
        }
    }
}

pub struct Mesh {
    pub num_vertices: u32,
    pub vertex_buffer: wgpu::Buffer,
    pub num_indices: u32,
    pub index_buffer: wgpu::Buffer
}

impl Mesh {
    pub(crate) fn new(device: &Device) -> Self {
        let num_vertices = VERTICES.len() as u32;
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(VERTICES),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(INDICES),
            usage: wgpu::BufferUsages::INDEX,
        });
        let num_indices = INDICES.len() as u32;
        Mesh {
            num_vertices,
            vertex_buffer,
            num_indices,
            index_buffer,
        }
    }
}

const VERTICES: &[Vertex] = &[
    Vertex { position: [-0.5, -0.5, -0.5], tex_coords: [0.0, 0.0], },
    Vertex { position: [0.5, -0.5, -0.5], tex_coords: [1.0, 0.0], },
    Vertex { position: [0.5, 0.5, -0.5], tex_coords: [1.0, 1.0], },
    Vertex { position: [-0.5, 0.5, -0.5], tex_coords: [0.0, 1.0], },

    Vertex { position: [-0.5, -0.5, 0.5], tex_coords: [0.0, 0.0], },
    Vertex { position: [0.5, -0.5, 0.5], tex_coords: [1.0, 0.0], },
    Vertex { position: [0.5, 0.5, 0.5], tex_coords: [1.0, 1.0], },
    Vertex { position: [-0.5, 0.5, 0.5], tex_coords: [0.0, 1.0], },
];

const INDICES: &[u16] = &[
    0, 2, 1,
    0, 3, 2,

    1, 2, 6,
    6, 5, 1,

    4, 5, 6,
    6, 7, 4,

    2, 3, 6,
    6, 3, 7,

    0, 7, 3,
    0, 4, 7,

    0, 1, 5,
    0, 5, 4
];

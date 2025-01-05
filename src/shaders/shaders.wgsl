// Vertex shader

struct CameraUniform {
    view_proj: mat4x4<f32>,
};

struct RotatorUniform {
    rotation: mat4x4<f32>,
};

@group(0) @binding(0)
var tree_texture: texture_2d<f32>;
@group(0) @binding(1)
var tree_texture_sampler: sampler;

@group(1) @binding(0)
var<uniform> camera: CameraUniform;

@group(2) @binding(0)
var<uniform> rotator: RotatorUniform;

@group(3) @binding(0)
var<storage, read> transformations: array<mat4x4<f32>>;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
    @builtin(instance_index) instance_index: u32
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>
};

@vertex
fn vs_main(
    vertex: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;
    let tr = transformations[vertex.instance_index];
    out.clip_position = camera.view_proj * tr * rotator.rotation * vec4<f32>(vertex.position, 1.0);
    out.tex_coords = vertex.tex_coords;
    return out;
}


@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return textureSample(tree_texture, tree_texture_sampler, in.tex_coords);
}
 
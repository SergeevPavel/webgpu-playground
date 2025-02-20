// Vertex shader

struct CameraUniform {
    view_proj: mat4x4<f32>,
};

struct RotatorUniform {
    rotation: mat4x4<f32>,
};

@group(0) @binding(0)
var t_diffuse: texture_2d<f32>;
@group(0) @binding(1)
var s_diffuse: sampler;

@group(1) @binding(0) // 1.
var<uniform> camera: CameraUniform;

@group(2) @binding(0)
var<uniform> rotator: RotatorUniform;

@group(3) @binding(0)
var<storage, read> transformations: array<mat4x4<f32>>;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
    @builtin(instance_index) instanceIndex: u32
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>
};

@vertex
fn vs_main(
    model: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;
    let tr = transformations[model.instanceIndex];
    out.clip_position = camera.view_proj * tr * rotator.rotation * vec4<f32>(model.position, 1.0);
    out.tex_coords = model.tex_coords;
    return out;
}


@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return textureSample(t_diffuse, s_diffuse, in.tex_coords);
}
 
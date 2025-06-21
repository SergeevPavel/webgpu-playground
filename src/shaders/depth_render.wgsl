

@group(0) @binding(0)
var depth_texture: texture_2d<f32>;
@group(0) @binding(1)
var depth_texture_sampler: sampler;

struct VertexOutput {
    @builtin(position) position: vec4f,
    @location(1) tex_coords: vec2<f32>
}

@vertex
fn depth_view_vs(@builtin(vertex_index) vertex_index : u32) -> VertexOutput {
      var pos = array(
        vec2(-1.0, -1.0),
        vec2( 1.0, -1.0),
        vec2(-1.0,  1.0),

        vec2( 1.0,  1.0),
        vec2(-1.0,  1.0),
        vec2( 1.0, -1.0),
      );

      var out: VertexOutput;

      out.position = vec4f(pos[vertex_index], 0, 1);
      out.tex_coords = pos[vertex_index];

      return out;
}

@fragment
fn depth_view_fs(in: VertexOutput) -> @location(0) vec4f {
    var x = textureSample(depth_texture, depth_texture_sampler, in.tex_coords).x;
    return vec4(0.0, 1.0, 0.0, x);
}


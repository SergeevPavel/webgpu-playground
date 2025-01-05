



@vertex
fn depth_view_vs(@builtin(vertex_index) vertex_index : u32) -> @builtin(position) vec4f {
      var pos = array(
        vec2(-1.0, -1.0),
        vec2( 1.0, -1.0),
        vec2(-1.0,  1.0),

        vec2( 1.0,  1.0),
        vec2(-1.0,  1.0),
        vec2( 1.0, -1.0),
      );

      return vec4f(pos[vertex_index], 0, 1);
}

@fragment
fn depth_view_fs() -> @location(0) vec4f {
    return vec4(0.0, 1.0, 0.0, 0.5);
}


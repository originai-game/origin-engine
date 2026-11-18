// Start the on-demand volume buffer with opaque linear depth. The exit
// pass writes RGB only, preserving this depth in A without another
// texture slot in the physical/lightmapped shaders.
@group(0) @binding(0) var opaque_depth: texture_2d<f32>;

@vertex
fn vs(@builtin(vertex_index) i: u32) -> @builtin(position) vec4<f32> {
  return fullscreen_clip(i);
}

@fragment
fn fs(@builtin(position) p: vec4<f32>) -> @location(0) vec4<u32> {
  return vec4<u32>(0u, 0u, 0u, bitcast<u32>(textureLoad(opaque_depth, vec2<i32>(p.xy), 0).r));
}

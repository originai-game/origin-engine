// A view's resolve (views.rs): its HDR image into a render target or a
// rect of the frame, exposed and tone mapped (`fs_color`), as linear
// radiance (`fs_linear`), or its depth as the normalised linear view depth
// (`fs_depth`): this part is shared, views_color.wgsl or views_depth.wgsl
// follows it.
struct Resolve {
  params: vec4<f32>,  // x: exposure multiplier, y: tone curve (0 ACES, 1 none, 2 AgX, 3 Khronos PBR Neutral), z: 1 mirrored across x, w: 1 encode sRGB here
  rect: vec4<f32>,    // xy: the rect's origin in destination pixels
  depth: vec4<f32>,   // x: near, y: far, z: 1 orthographic, w: source width
  hdr: vec4<f32>,     // display mapper (viewport only)
};
@group(0) @binding(1) var<uniform> r: Resolve;

struct VsOut { @builtin(position) pos: vec4<f32> };

@vertex
fn vs(@builtin(vertex_index) i: u32) -> VsOut {
  var o: VsOut;
  o.pos = fullscreen_clip(i);
  return o;
}

// The source texel of a destination pixel (mirrored across x for a mirror).
fn texel(pos: vec4<f32>) -> vec2<i32> {
  var p = vec2<i32>(pos.xy - r.rect.xy);
  if (r.params.z > 0.5) { p.x = i32(r.depth.w) - 1 - p.x; }
  return p;
}


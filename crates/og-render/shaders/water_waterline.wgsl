// The waterline across the lens (appended to water.wgsl).

// ── W6: the waterline ──
//
// Where the camera's near plane crosses the surface, each pixel is above or under the water by where its point on the
// near plane lies against the surface there (Crest's underwater curtain and meniscus, MIT, third_party/ocean/): the
// frame was drawn with the air's fog (camera.rs), and here what lies under the surface is fogged by the water over its
// depth, with a meniscus along the line between them, the thin, dark lens of water the surface draws up on a lens.

struct LineOut {
  @builtin(position) clip: vec4<f32>,
  @location(0) ndc: vec2<f32>,
};

@vertex
fn vs_waterline(@builtin(vertex_index) vi: u32) -> LineOut {
  let uv = vec2<f32>(f32((vi << 1u) & 2u), f32(vi & 2u));
  var o: LineOut;
  o.ndc = uv * 2.0 - 1.0;
  o.clip = vec4<f32>(o.ndc, 0.0, 1.0);
  return o;
}

// The surface's height (world y) at frame point q as the tick floats bodies on it (water.rs surface_height): over a
// sea, the rest point under q found as probe_sea finds it, at its finest spacing.
fn waterline_height(q: vec2<f32>) -> f32 {
  if (water.fft.x > 0.5) {
    var rest = q;
    for (var i = 0; i < 8; i++) {
      rest = q - ocean_displace(rest, water.levels[0].z).xz;
    }
    return water.model[3].y + ocean_displace(rest, water.levels[0].z).y;
  }
  let local = (water.inverse * vec4<f32>(q.x, water.model[3].y, q.y, 1.0)).xz;
  return (water.model * vec4<f32>(local.x, height_at(local), local.y, 1.0)).y;
}

@fragment
fn fs_waterline(in: LineOut) -> @location(0) vec4<f32> {
  let n4 = frame.inv_view_proj * vec4<f32>(in.ndc, 0.0, 1.0);
  let near = n4.xyz / n4.w;
  let s = near.y - waterline_height(near.xz);
  // Its distance from the line in pixels (the height's change across a pixel).
  let px = s / max(fwidthFine(s), 1e-7);
  let under = 1.0 - smoothstep(-0.5, 0.5, px);
  // What lies under it: fogged by the water over its distance (the depth prepass's view depth along the view's ray).
  let f4 = frame.inv_view_proj * vec4<f32>(in.ndc, 1.0, 1.0);
  let dir = normalize(f4.xyz / f4.w - near);
  let depth = textureLoad(scene_depth, depth_texel(in.clip.xy), 0).r;
  let dist = min(depth / max(dot(dir, frame.view_forward.xyz), 1e-3), 1e5);
  let fog = (1.0 - exp(-water.waterline.y * dist)) * under;
  var rgb = water.under.rgb * fog;
  var a = fog;
  // The meniscus: a dark band along the line, the water's own colour seen edge on through the lens of it.
  let w = water.waterline.z;
  if (w > 0.0) {
    let band = (1.0 - smoothstep(0.5 * w, w, abs(px))) * 0.85;
    rgb = mix(rgb, water.under.rgb * 0.25, band);
    a = mix(a, 1.0, band);
  }
  return vec4<f32>(rgb, a);
}

// McGuire et al., I3D 2012: TileMax, NeighborMax, line reconstruction.
// Separable reduction after Wojtek Pe's MIT implementation; provenance and
// license in third_party/motion-blur/. Velocities here are full centred
// shutter displacements in OUTPUT pixels, not UV or half displacements.
fn motion_tile_width() -> i32 {
  return i32(clamp(ceil(fxp.mb.z * 0.5), 1.0, max(fxp.screen.x, fxp.screen.y)));
}

fn motion_max(a: vec2<f32>, b: vec2<f32>) -> vec2<f32> {
  return select(a, b, dot(b, b) > dot(a, a));
}

fn motion_sweep(px: vec2<i32>) -> vec2<f32> {
  let size = vec2<i32>(textureDimensions(tex_gbuffer));
  let v = textureLoad(tex_gbuffer, prepass_px(px, size), 0).zw * fxp.screen.xy * fxp.mb.x;
  return v * min(1.0, fxp.mb.z / max(length(v), 1e-6));
}

@fragment
fn fs_motion_tile_x(in: VsOut) -> @location(0) vec4<f32> {
  let k = motion_tile_width();
  let base = vec2<i32>(in.pos.xy) * vec2<i32>(k, 1);
  var v = vec2<f32>(0.0);
  for (var x = 0; x < k && base.x + x < i32(fxp.screen.x); x++) {
    v = motion_max(v, motion_sweep(base + vec2<i32>(x, 0)));
  }
  return vec4<f32>(v, 0.0, 0.0);
}

@fragment
fn fs_motion_tile_y(in: VsOut) -> @location(0) vec4<f32> {
  let k = motion_tile_width();
  let base = vec2<i32>(in.pos.xy) * vec2<i32>(1, k);
  let size = vec2<i32>(textureDimensions(tex_a));
  var v = vec2<f32>(0.0);
  for (var y = 0; y < k && base.y + y < size.y; y++) {
    v = motion_max(v, textureLoad(tex_a, base + vec2<i32>(0, y), 0).xy);
  }
  return vec4<f32>(v, 0.0, 0.0);
}

@fragment
fn fs_motion_neighbor(in: VsOut) -> @location(0) vec4<f32> {
  let px = vec2<i32>(in.pos.xy);
  let size = vec2<i32>(textureDimensions(tex_a));
  var v = textureLoad(tex_a, px, 0).xy;
  for (var y = -1; y <= 1; y++) {
    for (var x = -1; x <= 1; x++) {
      let q = clamp(px + vec2<i32>(x, y), vec2<i32>(0), size - 1);
      v = motion_max(v, textureLoad(tex_a, q, 0).xy);
    }
  }
  return vec4<f32>(v, 0.0, 0.0);
}

@fragment
fn fs_motion_blur(in: VsOut) -> @location(0) vec4<f32> {
  let px = vec2<i32>(in.pos.xy);
  let c = textureLoad(tex_a, px, 0);
  var v = motion_sweep(px);
  let dominant = motion_max(textureLoad(tex_b, px / motion_tile_width(), 0).xy, v);
  if (dot(dominant, dominant) < 0.25) { return c; }
  let n = u32(fxp.mb.y);
  // NeighborMax supplies the direction, not another object's velocity.
  // Inspect sources along that line only when our own sweep is shorter.
  // A source must actually cover this pixel and be in front of it.
  if (dot(dominant, dominant) > dot(v, v)) {
    let z = depth_at(px);
    for (var i = 0u; i < n; i++) {
      let t = (f32(i) + 0.5) / f32(n) - 0.5;
      let q = vec2<i32>(floor(in.pos.xy + dominant * t));
      if (any(q < vec2<i32>(0)) || any(q >= vec2<i32>(fxp.screen.xy))) { continue; }
      let w = motion_sweep(q);
      if (dot(w, w) <= dot(v, v)) { continue; }
      let speed = length(w);
      let direction = w / max(speed, 1e-6);
      let delta = vec2<f32>(px - q);
      let along = dot(delta, direction);
      let across = length(delta - direction * along);
      if (abs(along) <= speed * 0.5 + 1.0 && across <= 1.0 && depth_at(q) <= z) { v = w; }
    }
  }
  if (dot(v, v) < 0.25) { return c; }
  let z0 = depth_at(px);
  let soft_z = max(z0 * 0.04, 1e-2);
  var sum = c.rgb;
  var weight = 1.0;
  for (var i = 0u; i < n; i++) {
    let t = (f32(i) + 0.5) / f32(n) - 0.5;
    let q_uv = in.uv + v * t * fxp.screen.zw;
    let q_z = depth_uv(q_uv);
    // Accept samples in front of or at similar depth to the centre, and
    // softly attenuate stationary closer foreground occluders.
    let w = clamp((q_z - z0) / soft_z + 1.0, 0.0, 1.0);
    sum += textureSampleLevel(tex_a, fx_linear, q_uv, 0.0).rgb * w;
    weight += w;
  }
  // Drawn as given (alpha below 1): no camera moved it.
  return vec4<f32>(mix(c.rgb, sum / max(weight, 1e-4), select(1.0, clamp(c.a, 0.0, 1.0), fxp.mb.w > 0.5)), c.a);
}

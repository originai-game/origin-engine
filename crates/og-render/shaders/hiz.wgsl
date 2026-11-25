// Conservative Reverse-Z Single-Pass Downsample (SPD) depth pyramid for GPU
// culling (cull_gpu.rs): each mip texel holds the farthest (minimum) depth of
// every texel it covers (including the 3rd row/column on odd dimensions).
//
// `base_spd` fuses copying 1x occluder depth into level 0 with up to 3 in-LDS
// 2x2 reductions (levels 0..3 in one dispatch for standard resolutions).
// `reduce_spd` performs an exact conservative odd-edge fold into `dst0` plus up
// to 3 in-LDS 2x2 reductions whenever intermediate levels have even (or 1) dims.
// `tail_spd` folds all remaining <= 16x16 mip levels (including odd dimensions)
// in one single workgroup.

@group(0) @binding(0) var<uniform> params: vec4<u32>; // x: active dst count (1..4)
@group(0) @binding(1) var src_tex: texture_2d<f32>;
@group(0) @binding(2) var dst0: texture_storage_2d<r32float, write>;
@group(0) @binding(3) var dst1: texture_storage_2d<r32float, write>;
@group(0) @binding(4) var dst2: texture_storage_2d<r32float, write>;
@group(0) @binding(5) var dst3: texture_storage_2d<r32float, write>;

var<workgroup> lds0: array<f32, 64>;
var<workgroup> lds1: array<f32, 16>;
var<workgroup> lds2: array<f32, 4>;
var<workgroup> lds_tail_a: array<f32, 256>;
var<workgroup> lds_tail_b: array<f32, 256>;

fn load_clamped(xy: vec2<u32>, dims: vec2<u32>) -> f32 {
  if (xy.x >= dims.x || xy.y >= dims.y) { return 1.0; }
  return textureLoad(src_tex, xy, 0).r;
}

@compute @workgroup_size(8, 8)
fn base_spd(@builtin(workgroup_id) wg: vec3<u32>, @builtin(local_invocation_id) lid: vec3<u32>) {
  let d0 = textureDimensions(dst0);
  let base = wg.xy * 16u + lid.xy * 2u;
  let a = load_clamped(base + vec2<u32>(0u, 0u), d0);
  let b = load_clamped(base + vec2<u32>(1u, 0u), d0);
  let c = load_clamped(base + vec2<u32>(0u, 1u), d0);
  let d = load_clamped(base + vec2<u32>(1u, 1u), d0);
  if (base.x < d0.x && base.y < d0.y) {
    textureStore(dst0, base, vec4<f32>(a, 0.0, 0.0, 1.0));
  }
  if (base.x + 1u < d0.x && base.y < d0.y) {
    textureStore(dst0, base + vec2<u32>(1u, 0u), vec4<f32>(b, 0.0, 0.0, 1.0));
  }
  if (base.x < d0.x && base.y + 1u < d0.y) {
    textureStore(dst0, base + vec2<u32>(0u, 1u), vec4<f32>(c, 0.0, 0.0, 1.0));
  }
  if (base.x + 1u < d0.x && base.y + 1u < d0.y) {
    textureStore(dst0, base + vec2<u32>(1u, 1u), vec4<f32>(d, 0.0, 0.0, 1.0));
  }
  let m1 = min(min(a, b), min(c, d));
  let p1 = wg.xy * 8u + lid.xy;
  let d1 = textureDimensions(dst1);
  if (params.x >= 2u && p1.x < d1.x && p1.y < d1.y) {
    textureStore(dst1, p1, vec4<f32>(m1, 0.0, 0.0, 1.0));
    lds0[lid.y * 8u + lid.x] = m1;
  } else {
    lds0[lid.y * 8u + lid.x] = 1.0;
  }
  workgroupBarrier();

  if (params.x >= 3u && lid.x < 4u && lid.y < 4u) {
    let i = lid.y * 16u + lid.x * 2u;
    let m2 = min(min(lds0[i], lds0[i + 1u]), min(lds0[i + 8u], lds0[i + 9u]));
    let p2 = wg.xy * 4u + lid.xy;
    let d2 = textureDimensions(dst2);
    if (p2.x < d2.x && p2.y < d2.y) {
      textureStore(dst2, p2, vec4<f32>(m2, 0.0, 0.0, 1.0));
      lds1[lid.y * 4u + lid.x] = m2;
    } else {
      lds1[lid.y * 4u + lid.x] = 1.0;
    }
  }
  workgroupBarrier();

  if (params.x >= 4u && lid.x < 2u && lid.y < 2u) {
    let i = lid.y * 8u + lid.x * 2u;
    let m3 = min(min(lds1[i], lds1[i + 1u]), min(lds1[i + 4u], lds1[i + 5u]));
    let p3 = wg.xy * 2u + lid.xy;
    let d3 = textureDimensions(dst3);
    if (p3.x < d3.x && p3.y < d3.y) {
      textureStore(dst3, p3, vec4<f32>(m3, 0.0, 0.0, 1.0));
    }
  }
}

@compute @workgroup_size(8, 8)
fn reduce_spd(@builtin(workgroup_id) wg: vec3<u32>, @builtin(local_invocation_id) lid: vec3<u32>) {
  let gid = wg.xy * 8u + lid.xy;
  let d0 = textureDimensions(dst0);
  var depth = 1.0;
  if (gid.x < d0.x && gid.y < d0.y) {
    let below = textureDimensions(src_tex, 0);
    let x0 = gid.x * 2u;
    let y0 = gid.y * 2u;
    // An odd size below folds its last row or column into the last texel.
    let x1 = select(x0 + 1u, below.x - 1u, gid.x == d0.x - 1u);
    let y1 = select(y0 + 1u, below.y - 1u, gid.y == d0.y - 1u);
    for (var y = y0; y <= y1; y++) {
      for (var x = x0; x <= x1; x++) {
        depth = min(depth, textureLoad(src_tex, vec2<u32>(x, y), 0).r);
      }
    }
    textureStore(dst0, gid, vec4<f32>(depth, 0.0, 0.0, 1.0));
  }
  lds0[lid.y * 8u + lid.x] = depth;
  workgroupBarrier();

  if (params.x >= 2u && lid.x < 4u && lid.y < 4u) {
    let i = lid.y * 16u + lid.x * 2u;
    let m1 = min(min(lds0[i], lds0[i + 1u]), min(lds0[i + 8u], lds0[i + 9u]));
    let p1 = wg.xy * 4u + lid.xy;
    let d1 = textureDimensions(dst1);
    if (p1.x < d1.x && p1.y < d1.y) {
      textureStore(dst1, p1, vec4<f32>(m1, 0.0, 0.0, 1.0));
      lds1[lid.y * 4u + lid.x] = m1;
    } else {
      lds1[lid.y * 4u + lid.x] = 1.0;
    }
  }
  workgroupBarrier();

  if (params.x >= 3u && lid.x < 2u && lid.y < 2u) {
    let i = lid.y * 8u + lid.x * 2u;
    let m2 = min(min(lds1[i], lds1[i + 1u]), min(lds1[i + 4u], lds1[i + 5u]));
    let p2 = wg.xy * 2u + lid.xy;
    let d2 = textureDimensions(dst2);
    if (p2.x < d2.x && p2.y < d2.y) {
      textureStore(dst2, p2, vec4<f32>(m2, 0.0, 0.0, 1.0));
      lds2[lid.y * 2u + lid.x] = m2;
    } else {
      lds2[lid.y * 2u + lid.x] = 1.0;
    }
  }
  workgroupBarrier();

  if (params.x >= 4u && lid.x == 0u && lid.y == 0u) {
    let m3 = min(min(lds2[0], lds2[1]), min(lds2[2], lds2[3]));
    let d3 = textureDimensions(dst3);
    if (wg.x < d3.x && wg.y < d3.y) {
      textureStore(dst3, wg.xy, vec4<f32>(m3, 0.0, 0.0, 1.0));
    }
  }
}

fn tail_reduce_from_a(xy: vec2<u32>, nw: u32, nh: u32, cw: u32, ch: u32) -> f32 {
  let x0 = xy.x * 2u;
  let y0 = xy.y * 2u;
  let x1 = select(x0 + 1u, cw - 1u, xy.x == nw - 1u);
  let y1 = select(y0 + 1u, ch - 1u, xy.y == nh - 1u);
  var d = 1.0;
  for (var y = y0; y <= y1; y++) {
    for (var x = x0; x <= x1; x++) {
      d = min(d, lds_tail_a[y * 16u + x]);
    }
  }
  return d;
}

fn tail_reduce_from_b(xy: vec2<u32>, nw: u32, nh: u32, cw: u32, ch: u32) -> f32 {
  let x0 = xy.x * 2u;
  let y0 = xy.y * 2u;
  let x1 = select(x0 + 1u, cw - 1u, xy.x == nw - 1u);
  let y1 = select(y0 + 1u, ch - 1u, xy.y == nh - 1u);
  var d = 1.0;
  for (var y = y0; y <= y1; y++) {
    for (var x = x0; x <= x1; x++) {
      d = min(d, lds_tail_b[y * 16u + x]);
    }
  }
  return d;
}

@compute @workgroup_size(8, 8)
fn tail_spd(@builtin(local_invocation_id) lid: vec3<u32>) {
  let s0 = textureDimensions(src_tex, 0);
  // Load entire <= 16x16 source mip into lds_tail_a.
  for (var dy = 0u; dy < 2u; dy++) {
    for (var dx = 0u; dx < 2u; dx++) {
      let p = vec2<u32>(lid.x + dx * 8u, lid.y + dy * 8u);
      var v = 1.0;
      if (p.x < s0.x && p.y < s0.y) {
        v = textureLoad(src_tex, p, 0).r;
      }
      lds_tail_a[p.y * 16u + p.x] = v;
    }
  }
  workgroupBarrier();

  let w1 = max(s0.x >> 1u, 1u);
  let h1 = max(s0.y >> 1u, 1u);
  if (lid.x < w1 && lid.y < h1) {
    let v = tail_reduce_from_a(lid.xy, w1, h1, s0.x, s0.y);
    textureStore(dst0, lid.xy, vec4<f32>(v, 0.0, 0.0, 1.0));
    lds_tail_b[lid.y * 16u + lid.x] = v;
  }
  workgroupBarrier();

  let w2 = max(w1 >> 1u, 1u);
  let h2 = max(h1 >> 1u, 1u);
  if (params.x >= 2u && lid.x < w2 && lid.y < h2) {
    let v = tail_reduce_from_b(lid.xy, w2, h2, w1, h1);
    textureStore(dst1, lid.xy, vec4<f32>(v, 0.0, 0.0, 1.0));
    lds_tail_a[lid.y * 16u + lid.x] = v;
  }
  workgroupBarrier();

  let w3 = max(w2 >> 1u, 1u);
  let h3 = max(h2 >> 1u, 1u);
  if (params.x >= 3u && lid.x < w3 && lid.y < h3) {
    let v = tail_reduce_from_a(lid.xy, w3, h3, w2, h2);
    textureStore(dst2, lid.xy, vec4<f32>(v, 0.0, 0.0, 1.0));
    lds_tail_b[lid.y * 16u + lid.x] = v;
  }
  workgroupBarrier();

  let w4 = max(w3 >> 1u, 1u);
  let h4 = max(h3 >> 1u, 1u);
  if (params.x >= 4u && lid.x < w4 && lid.y < h4) {
    let v = tail_reduce_from_b(lid.xy, w4, h4, w3, h3);
    textureStore(dst3, lid.xy, vec4<f32>(v, 0.0, 0.0, 1.0));
  }
}

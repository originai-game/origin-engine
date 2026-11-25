// fxpost.wgsl's AO blur (`fs_ao_blur`) at full resolution: the same centred
// 3x3 normal- and depth-aware taps read via gathers instead of 27 loads,
// matching fs_ao_blur's loop order to the bit. Half-resolution AO keeps
// fs_ao_blur (its depth is at twice its texel). Appended to the effect
// module (fullscreen.wgsl, fxpost.wgsl, ssgi.wgsl; post.rs `fx_source`).

// Nearest, clamped to the edge: a gather's out-of-range texels are the
// edge's, as fs_ao_blur's clamped loads read them.
@group(0) @binding(6) var fx_nearest: sampler;

// One tap of the blur: (sum, weight) with the tap of depth `d`, raw AO `a`,
// and oct-encoded world normal `gn` added, as fs_ao_blur's loop body adds it.
fn ao_blur_tap(acc: vec2<f32>, d: f32, a: f32, gn: vec2<f32>, d0: f32, n0: vec3<f32>, scale: f32) -> vec2<f32> {
  let facing = max(dot(oct_decode(gn), n0), 0.0);
  let w = exp(-abs(d - d0) / scale) * facing * facing * facing * facing + 1e-6;
  return vec2<f32>(acc.x + a * w, acc.y + w);
}

@fragment
fn fs_ao_blur_gather(in: VsOut) -> @location(0) vec4<f32> {
  let px = vec2<i32>(in.pos.xy);
  if (fxp.cam.w > 0.5 && fxp.cam.w < 1.5) { return textureLoad(tex_b, px, 0); }
  let term = max(fxp.ao2.z, 1.0);
  if (fxp.cam.w > 1.5 || fxp.ao2.y > 0.5) {
    return vec4<f32>(min(textureLoad(tex_b, px, 0).r * term, 1.0), 0.0, 0.0, 1.0);
  }
  let d0 = depth_at(px);
  if (ao_blur_sky(d0)) { return vec4<f32>(1.0, 0.0, 0.0, 1.0); }
  let n0 = gbuffer_normal(px);
  let scale = max(d0 * 0.04, 1e-3);
  // The corner between texels px - 1 and px: offset (0,0) covers
  // [px-1..px, px-1..px]; offsets (1,0), (0,1), (1,1) reach the remaining
  // texels of the centred 3x3 [px-1..px+1]. Each gather is
  // (bottom-left, bottom-right, top-right, top-left) of its 2x2 block.
  let c = vec2<f32>(px) / vec2<f32>(textureDimensions(tex_b));
  let d00 = textureGather(0, tex_depth, fx_nearest, c);
  let d10 = textureGather(0, tex_depth, fx_nearest, c, vec2<i32>(1, 0));
  let d01 = textureGather(0, tex_depth, fx_nearest, c, vec2<i32>(0, 1));
  let d11 = textureGather(0, tex_depth, fx_nearest, c, vec2<i32>(1, 1));
  let a00 = textureGather(0, tex_b, fx_nearest, c);
  let a10 = textureGather(0, tex_b, fx_nearest, c, vec2<i32>(1, 0));
  let a01 = textureGather(0, tex_b, fx_nearest, c, vec2<i32>(0, 1));
  let a11 = textureGather(0, tex_b, fx_nearest, c, vec2<i32>(1, 1));
  let nx00 = textureGather(0, tex_gbuffer, fx_nearest, c);
  let nx10 = textureGather(0, tex_gbuffer, fx_nearest, c, vec2<i32>(1, 0));
  let nx01 = textureGather(0, tex_gbuffer, fx_nearest, c, vec2<i32>(0, 1));
  let nx11 = textureGather(0, tex_gbuffer, fx_nearest, c, vec2<i32>(1, 1));
  let ny00 = textureGather(1, tex_gbuffer, fx_nearest, c);
  let ny10 = textureGather(1, tex_gbuffer, fx_nearest, c, vec2<i32>(1, 0));
  let ny01 = textureGather(1, tex_gbuffer, fx_nearest, c, vec2<i32>(0, 1));
  let ny11 = textureGather(1, tex_gbuffer, fx_nearest, c, vec2<i32>(1, 1));
  var acc = vec2<f32>(0.0);
  // Row y = -1: (-1,-1), (0,-1), (+1,-1)
  acc = ao_blur_tap(acc, d00.w, a00.w, vec2<f32>(nx00.w, ny00.w), d0, n0, scale);
  acc = ao_blur_tap(acc, d00.z, a00.z, vec2<f32>(nx00.z, ny00.z), d0, n0, scale);
  acc = ao_blur_tap(acc, d10.z, a10.z, vec2<f32>(nx10.z, ny10.z), d0, n0, scale);
  // Row y = 0: (-1,0), (0,0), (+1,0)
  acc = ao_blur_tap(acc, d00.x, a00.x, vec2<f32>(nx00.x, ny00.x), d0, n0, scale);
  acc = ao_blur_tap(acc, d00.y, a00.y, vec2<f32>(nx00.y, ny00.y), d0, n0, scale);
  acc = ao_blur_tap(acc, d10.y, a10.y, vec2<f32>(nx10.y, ny10.y), d0, n0, scale);
  // Row y = +1: (-1,+1), (0,+1), (+1,+1)
  acc = ao_blur_tap(acc, d01.x, a01.x, vec2<f32>(nx01.x, ny01.x), d0, n0, scale);
  acc = ao_blur_tap(acc, d01.y, a01.y, vec2<f32>(nx01.y, ny01.y), d0, n0, scale);
  acc = ao_blur_tap(acc, d11.y, a11.y, vec2<f32>(nx11.y, ny11.y), d0, n0, scale);
  // GTAO's raw term is stored over fxp.ao2.z (fxpost.wgsl fs_gtao).
  return vec4<f32>(min(acc.x / max(acc.y, 1e-5) * term, 1.0), 0.0, 0.0, 1.0);
}

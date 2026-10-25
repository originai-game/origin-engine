// fxpost.wgsl's AO blur (`fs_ao_blur`) on the full tier: the same 4x4
// taps of view depth and raw AO read as four gathers each instead of 32
// loads (a gather costs the texture unit what one load does), then
// weighed and summed in the loop's order (rows top to bottom, each left to
// right), so its answer is fs_ao_blur's to the bit. GLSL ES 3.0 has no
// gather: WebGL2 keeps fs_ao_blur. Appended to the effect module
// (fullscreen.wgsl, fxpost.wgsl, ssr.wgsl) on the full tier only (post.rs
// `fx_source`).

// Nearest, clamped to the edge: a gather's out-of-range texels are the
// edge's, as fs_ao_blur's clamped loads read them.
@group(0) @binding(6) var fx_nearest: sampler;

// One tap of the blur: (sum, weight) with the tap of depth `d` and raw AO
// `a` added, as fs_ao_blur's loop body adds it.
fn ao_blur_tap(acc: vec2<f32>, d: f32, a: f32, d0: f32, scale: f32) -> vec2<f32> {
  let w = exp(-abs(d - d0) / scale);
  return vec2<f32>(acc.x + a * w, acc.y + w);
}

@fragment
fn fs_ao_blur_gather(in: VsOut) -> @location(0) vec4<f32> {
  let px = vec2<i32>(in.pos.xy);
  let d0 = depth_at(px);
  if (ao_blur_sky(d0)) { return vec4<f32>(1.0, 0.0, 0.0, 1.0); }
  let scale = max(d0 * 0.04, 1e-3);
  // The corner between texels px - 2 and px - 1: the gather there reads
  // the 2x2 block from px - 2 (a 2x2 gather reads the texels around its
  // coordinate); offsets of 2 reach the other three blocks. Each gather is
  // (bottom left, bottom right, top right, top left) of its block.
  let c = (vec2<f32>(px) - 1.0) / vec2<f32>(textureDimensions(tex_b));
  let d00 = textureGather(0, tex_depth, fx_nearest, c);
  let d10 = textureGather(0, tex_depth, fx_nearest, c, vec2<i32>(2, 0));
  let d01 = textureGather(0, tex_depth, fx_nearest, c, vec2<i32>(0, 2));
  let d11 = textureGather(0, tex_depth, fx_nearest, c, vec2<i32>(2, 2));
  let a00 = textureGather(0, tex_b, fx_nearest, c);
  let a10 = textureGather(0, tex_b, fx_nearest, c, vec2<i32>(2, 0));
  let a01 = textureGather(0, tex_b, fx_nearest, c, vec2<i32>(0, 2));
  let a11 = textureGather(0, tex_b, fx_nearest, c, vec2<i32>(2, 2));
  var acc = vec2<f32>(0.0);
  // Row y = -2, then -1, 0, 1.
  acc = ao_blur_tap(acc, d00.w, a00.w, d0, scale);
  acc = ao_blur_tap(acc, d00.z, a00.z, d0, scale);
  acc = ao_blur_tap(acc, d10.w, a10.w, d0, scale);
  acc = ao_blur_tap(acc, d10.z, a10.z, d0, scale);
  acc = ao_blur_tap(acc, d00.x, a00.x, d0, scale);
  acc = ao_blur_tap(acc, d00.y, a00.y, d0, scale);
  acc = ao_blur_tap(acc, d10.x, a10.x, d0, scale);
  acc = ao_blur_tap(acc, d10.y, a10.y, d0, scale);
  acc = ao_blur_tap(acc, d01.w, a01.w, d0, scale);
  acc = ao_blur_tap(acc, d01.z, a01.z, d0, scale);
  acc = ao_blur_tap(acc, d11.w, a11.w, d0, scale);
  acc = ao_blur_tap(acc, d11.z, a11.z, d0, scale);
  acc = ao_blur_tap(acc, d01.x, a01.x, d0, scale);
  acc = ao_blur_tap(acc, d01.y, a01.y, d0, scale);
  acc = ao_blur_tap(acc, d11.x, a11.x, d0, scale);
  acc = ao_blur_tap(acc, d11.y, a11.y, d0, scale);
  return vec4<f32>(acc.x / max(acc.y, 1e-5), 0.0, 0.0, 1.0);
}

// FSR 3.1's RCAS (ffx_fsr3upscaler_rcas.h over FSR1's FsrRcasF with
// FSR_RCAS_DENOISE; MIT, third_party/fidelityfx/LICENSE.txt): robust
// contrast-adaptive sharpening of the accumulated output, in exposed light,
// limited so no pixel leaves the range its ring of four allows. It writes
// its own target; the history keeps the unsharpened image. The alpha is
// passed through. Prepended by fullscreen.wgsl and fsr_common.wgsl.

@group(0) @binding(1) var rcas_input: texture_2d<f32>;
@group(0) @binding(2) var<storage, read> frame_info: vec4<f32>;
@group(0) @binding(3) var rcas_output: texture_storage_2d<rgba16float, write>;

const RCAS_LIMIT: f32 = 0.25 - 1.0 / 16.0;

fn exposure() -> f32 { return exposure_of(frame_info); }

fn load(p: vec2<i32>) -> vec4<f32> {
  let c = textureLoad(rcas_input, clamp(p, vec2<i32>(0), upscale_size() - 1), 0);
  return vec4<f32>(c.rgb * exposure(), c.a);
}

fn max3(a: f32, b: f32, c: f32) -> f32 { return max(a, max(b, c)); }
fn min3(a: f32, b: f32, c: f32) -> f32 { return min(a, min(b, c)); }

@compute @workgroup_size(8, 8, 1)
fn cs(@builtin(global_invocation_id) id: vec3<u32>) {
  let sp = vec2<i32>(id.xy);
  if (!is_on_screen(sp, upscale_size())) { return; }
  //    b
  //  d e f
  //    h
  let b = load(sp + vec2<i32>(0, -1)).rgb;
  let d = load(sp + vec2<i32>(-1, 0)).rgb;
  let ee = load(sp);
  let e = ee.rgb;
  let f = load(sp + vec2<i32>(1, 0)).rgb;
  let h = load(sp + vec2<i32>(0, 1)).rgb;
  // Luma times 2.
  let bl = b.b * 0.5 + (b.r * 0.5 + b.g);
  let dl = d.b * 0.5 + (d.r * 0.5 + d.g);
  let el = e.b * 0.5 + (e.r * 0.5 + e.g);
  let fl = f.b * 0.5 + (f.r * 0.5 + f.g);
  let hl = h.b * 0.5 + (h.r * 0.5 + h.g);
  // Noise detection.
  var nz = 0.25 * bl + 0.25 * dl + 0.25 * fl + 0.25 * hl - el;
  let range = max3(max3(bl, dl, el), fl, hl) - min3(min3(bl, dl, el), fl, hl);
  nz = saturate(abs(nz) / max(range, 1e-30));
  nz = -0.5 * nz + 1.0;
  // Min and max of the ring.
  let mn4 = min(min(min(b, d), f), h);
  let mx4 = max(max(max(b, d), f), h);
  let peak = vec2<f32>(1.0, -4.0);
  let lower = saturate(el / max(min(min3(bl, dl, fl), hl), 1e-30));
  let hit_min = mn4 / (4.0 * mx4) * lower;
  let hit_max = (peak.x - mx4) / (4.0 * mn4 + peak.y);
  let lobe_rgb = max(-hit_min, hit_max);
  var lobe = max(-RCAS_LIMIT, min(max3(lobe_rgb.r, lobe_rgb.g, lobe_rgb.b), 0.0)) * fsr.extra.w;
  lobe *= nz;
  let rcp_l = 1.0 / (4.0 * lobe + 1.0);
  let pix = (lobe * b + lobe * d + lobe * h + lobe * f + e) * rcp_l;
  textureStore(rcas_output, sp, vec4<f32>(pix / exposure(), ee.a));
}

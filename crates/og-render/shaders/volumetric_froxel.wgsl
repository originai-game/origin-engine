// The froxel grid of volumetric.wgsl (full tier): inject, integrate, apply.

@group(1) @binding(1) var froxel_out: texture_storage_3d<rgba16float, write>;

// Depth samples a froxel's light is the mean of. A slice is up to a few
// metres deep (quadratic slices to `distance`) and the sun's shadow can
// change inside it many times (a shaft through a slot, the slots' shadows
// beside it): a single sample at the slice's middle lands in or out of a
// shaft depending on the column, which the front-to-back sum turns into
// vertical streaks. INJECT_TAPS stratified samples over the slice's depth,
// each jittered within its stratum per froxel and per frame, integrate the
// slice instead.
const INJECT_TAPS: u32 = 8u;

// Interleaved gradient noise (Jimenez 2014) at a froxel, moved by the
// frame's seed (vol.grid.w) so successive frames take other strata.
fn froxel_noise(id: vec3<u32>) -> f32 {
  let p = vec2<f32>(id.xy) + vec2<f32>(5.588238, 3.1415) * f32(id.z) + vec2<f32>(47.0, 17.0) * vol.grid.w;
  return fract(52.9829189 * fract(dot(p, vec2<f32>(0.06711056, 0.00583715))));
}

@compute @workgroup_size(8, 8, 1)
fn inject(@builtin(global_invocation_id) id: vec3<u32>) {
  let grid = vec3<u32>(vol.grid.xyz);
  if (id.x >= grid.x || id.y >= grid.y || id.z >= grid.z) { return; }
  let uv = (vec2<f32>(id.xy) + 0.5) / vol.grid.xy;
  let dir = view_ray(uv);
  let stretch = 1.0 / max(dot(dir, frame.view_forward.xyz), 1e-3);
  let jitter = froxel_noise(id);
  var sigma_sum = 0.0;
  var light_sum = vec3<f32>(0.0);
  for (var t = 0u; t < INJECT_TAPS; t++) {
    let f = (f32(id.z) + (f32(t) + jitter) / f32(INJECT_TAPS)) / vol.grid.z;
    let p = frame.view_pos.xyz + dir * slice_depth(f) * stretch;
    let sigma = vol_density(p);
    sigma_sum += sigma;
    if (sigma > 0.0) {
      light_sum += sigma * (sun_in(p, dir) + locals_in(p, dir, uv * vol.extent.zw));
    }
  }
  let n = f32(INJECT_TAPS);
  textureStore(froxel_out, id, vec4<f32>(vol.scatter.rgb * light_sum / n, sigma_sum / n));
}

@group(1) @binding(2) var froxel_in: texture_3d<f32>;
@group(1) @binding(3) var froxel_sum: texture_storage_3d<rgba16float, write>;

@compute @workgroup_size(8, 8, 1)
fn integrate(@builtin(global_invocation_id) id: vec3<u32>) {
  let grid = vec3<u32>(vol.grid.xyz);
  if (id.x >= grid.x || id.y >= grid.y) { return; }
  let uv = (vec2<f32>(id.xy) + 0.5) / vol.grid.xy;
  let stretch = 1.0 / max(dot(view_ray(uv), frame.view_forward.xyz), 1e-3);
  var acc = vec3<f32>(0.0);
  var transmittance = 1.0;
  var near = 0.0;
  for (var z = 0u; z < grid.z; z++) {
    let far = slice_depth(f32(z + 1u) / vol.grid.z);
    let thick = (far - near) * stretch;
    near = far;
    let s = textureLoad(froxel_in, vec3<i32>(vec3<u32>(id.xy, z)), 0);
    let tr = exp(-s.a * thick);
    acc += transmittance * s.rgb * select(thick, (1.0 - tr) / s.a, s.a > 1e-5);
    transmittance *= tr;
    textureStore(froxel_sum, vec3<u32>(id.xy, z), vec4<f32>(acc, transmittance));
  }
}

@group(1) @binding(5) var froxel_tex: texture_3d<f32>;
@group(1) @binding(6) var froxel_sampler: sampler;

@fragment
fn fs_froxel(in: VolOut) -> @location(0) vec4<f32> {
  let d = min(pixel_depth(in.pos.xy), vol.extent.x);
  let uv = in.pos.xy / vol.extent.zw;
  let f = sqrt(max(d, 0.0) / vol.extent.x);
  let z = f - 0.5 / vol.grid.z;
  let s = textureSampleLevel(froxel_tex, froxel_sampler, vec3<f32>(uv, max(z, 0.5 / vol.grid.z)), 0.0);
  // Nearer than the first slice's far edge: that slice's share of it.
  let k = clamp(f * vol.grid.z, 0.0, 1.0);
  return mix(vec4<f32>(0.0, 0.0, 0.0, 1.0), s, k);
}


// The froxel grid of volumetric.wgsl (VolumetricMethod::Froxel): inject, integrate, apply.

@group(1) @binding(1) var froxel_out: texture_storage_3d<rgba16float, write>;
@group(1) @binding(7) var froxel_history: texture_3d<f32>;
@group(1) @binding(8) var froxel_hist_sampler: sampler;

// Temporal froxel injection (Wronski 2014 / Hillaire 2015): 2 stratified
// jittered depth samples per froxel blended via 90% temporal reprojection EMA
// cuts shadow & cluster lookups 4x while converging smoothly across frames.
const INJECT_TAPS: u32 = 2u;

// Interleaved gradient noise (Jimenez 2014) at a froxel, moved by the
// frame's seed (vol.grid.w) so successive frames take other strata.
fn froxel_noise(id: vec3<u32>) -> f32 {
  let p = vec2<f32>(id.xy) + vec2<f32>(5.588238, 3.1415) * f32(id.z) + vec2<f32>(47.0, 17.0) * vol.grid.w;
  return ign(p);
}

@compute @workgroup_size(8, 8, 1)
fn inject(@builtin(global_invocation_id) id: vec3<u32>) {
  let grid = vec3<u32>(vol.grid.xyz);
  if (id.x >= grid.x || id.y >= grid.y || id.z >= grid.z) { return; }
  let uv = (vec2<f32>(id.xy) + 0.5) / vol.grid.xy;
  let dir = view_ray(uv);
  let stretch = 1.0 / max(dot(dir, frame.view_forward.xyz), 1e-3);
  let jitter = froxel_noise(id);
  // Lateral froxel width for caustic focus filtering (never clamped by ray-axis
  // slice thickness `deep`, which previously blurred underwater light shafts away).
  let across = length(view_ray(uv + vec2<f32>(1.0 / vol.grid.x, 0.0)) - dir);
  let deep = (slice_depth(f32(id.z + 1u) / vol.grid.z) - slice_depth(f32(id.z) / vol.grid.z)) * stretch / f32(INJECT_TAPS);
  var sigma_sum = 0.0;
  var light_sum = vec3<f32>(0.0);
  var own_sum = vec3<f32>(0.0);
  // The fog volumes' share of what the slice scatters, for the environment's light: that changes slowly, so the
  // froxel takes it once, at its middle.
  var local_sum = vec3<f32>(0.0);
  for (var t = 0u; t < INJECT_TAPS; t++) {
    let f = (f32(id.z) + (f32(t) + jitter) / f32(INJECT_TAPS)) / vol.grid.z;
    let along = slice_depth(f) * stretch;
    let p = frame.view_pos.xyz + dir * along;
    let edges = (vec2<f32>(f32(id.z)) + vec2<f32>(f32(t), f32(t + 1u)) / f32(INJECT_TAPS)) / vol.grid.z;
    let span = vec2<f32>(slice_depth(edges.x), slice_depth(edges.y)) * stretch;
    let m = vol_medium(p, dir, vec3<f32>(span, deep));
    sigma_sum += m.sigma;
    if (m.sigma > 0.0) {
      local_sum += m.local;
      own_sum += m.own;
      light_sum += medium_in(p, dir, uv * vol.extent.zw, 0xffffffffu, along * across, m, vec3<f32>(0.0), false);
    }
  }
  let mid_depth = slice_depth((f32(id.z) + 0.5) / vol.grid.z);
  let middle = frame.view_pos.xyz + dir * (mid_depth * stretch);
  // Modulate global fog ambient (`own`) by Probe GI sky visibility when active
  // so indoor / under-canopy volumetric fog does not self-glow.
  if (own_sum.x + own_sum.y + own_sum.z > 0.0) {
    if (!VOL_WATER && OG_GI && probe_data.v[GI_HEADER_AT].x >= 0.5) {
      let env = env_in(middle);
      let amb = ambient_in(middle, env);
      let sky_vis = clamp(luma(amb) / max(luma(env), 1e-4), 0.15, 1.0);
      light_sum += own_sum * sky_vis;
    } else {
      light_sum += own_sum;
    }
  }
  if (local_sum.x + local_sum.y + local_sum.z > 0.0) {
    light_sum += local_sum * ambient_in(middle, env_in(middle));
  }
  let n = f32(INJECT_TAPS);
  var sample = vec4<f32>(light_sum / n, sigma_sum / n);
  if (vol.local.w > vol.local.z) {
    // A finite-distance clear ramp has unbounded extinction at its end.
    // Store the finite column, not extinction per metre, in the half-float grid.
    sample *= deep * n;
  }
  // Temporal reprojection EMA against previous frame's froxel scatter volume.
  if (vol.temporal.x > 0.0) {
    let prev_clip = vol.prev_view_proj * vec4<f32>(middle, 1.0);
    if (prev_clip.w > 1e-4) {
      let prev_ndc = prev_clip.xy / prev_clip.w;
      let prev_uv = vec2<f32>(0.5 + 0.5 * prev_ndc.x, 0.5 - 0.5 * prev_ndc.y);
      let prev_z = sqrt(clamp(prev_clip.w / max(vol.temporal.y, 1e-3), 0.0, 1.0));
      if (all(prev_uv >= vec2<f32>(0.0)) && all(prev_uv <= vec2<f32>(1.0)) && prev_z > 0.0 && prev_z < 1.0) {
        let hist = textureSampleLevel(froxel_history, froxel_hist_sampler, vec3<f32>(prev_uv, prev_z), 0.0);
        sample = mix(sample, hist, vol.temporal.x);
      }
    }
  }
  textureStore(froxel_out, id, sample);
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
    var tr = exp(-s.a * thick);
    if (vol.local.w > vol.local.z) {
      tr = exp(-s.a);
      acc += transmittance * s.rgb * select(1.0, (1.0 - tr) / s.a, s.a > 1e-5);
    } else {
      acc += transmittance * s.rgb * select(thick, (1.0 - tr) / s.a, s.a > 1e-5);
    }
    transmittance *= tr;
    textureStore(froxel_sum, vec3<u32>(id.xy, z), vec4<f32>(acc, transmittance));
  }
}

@group(1) @binding(5) var froxel_tex: texture_3d<f32>;
@group(1) @binding(6) var froxel_sampler: sampler;

fn froxel_at(uv: vec2<f32>, d: f32) -> vec4<f32> {
  let f = sqrt(max(d, 0.0) / vol.extent.x);
  let z = f - 0.5 / vol.grid.z;
  let s = textureSampleLevel(froxel_tex, froxel_sampler, vec3<f32>(uv, max(z, 0.5 / vol.grid.z)), 0.0);
  // Nearer than the first slice's far edge: that slice's share of it.
  let k = clamp(f * vol.grid.z, 0.0, 1.0);
  return mix(vec4<f32>(0.0, 0.0, 0.0, 1.0), s, k);
}

// Samples each spot's light takes along a pixel ray (`spots_in`): equi-angular
// stratified + temporally IGN-jittered, so 8 taps reconstruct spot cones cleanly.
const SPOT_TAPS: u32 = 8u;

// Each spot's light scattered towards the eye along the pixel ray, inside
// its finite cone only: a cone can be narrower than a froxel, so neither a
// centre ray nor the grid can draw its silhouette. The grid supplies the
// extinction; no spot light is stored in it.
//
// The samples are equi-angular in the light's view of the ray (Kulla and
// Fajardo, "Importance Sampling Techniques for Path Tracing in Participating
// Media", EGSR 2012): their density along the ray follows the inverse square
// of the distance to the light, which its falloff divides out, so a few
// samples where the light is near do what many evenly spread ones did. The
// shadow, cone, cookie and medium vary along the ray; the samples are
// stratified in angle, jittered per pixel and per frame.
fn spots_in(uv: vec2<f32>, d: f32) -> vec3<f32> {
  var sum = vec3<f32>(0.0);
  if (VOL_WATER || frame.params.z < 0.5 || vol.temporal.z < 0.5) { return sum; }
  let dir = view_ray(uv);
  let forward = max(dot(dir, frame.view_forward.xyz), 1e-3);
  let jitter = ign(uv * vol.extent.zw + vec2<f32>(47.0, 17.0) * vol.grid.w);
  for (var i = 0u; i < u32(frame.params.z); i++) {
    let light = light_at(i);
    if (light.color_kind.w < 0.5) { continue; }
    let span = spot_span(light, frame.view_pos.xyz, dir, vec2<f32>(0.0, d / forward));
    if (span.y <= span.x) { continue; }
    // The ray's nearest approach to the light (at `mid`, `reach` from it)
    // and the angles the span's ends make there.
    let to_light = light.position_range.xyz - frame.view_pos.xyz;
    let mid = dot(to_light, dir);
    let reach = max(length(to_light - dir * mid), 1e-4);
    let a0 = atan2(span.x - mid, reach);
    let a1 = atan2(span.y - mid, reach);
    // A sample's share of the span: 1 / (SPOT_TAPS pdf), the pdf being
    // reach / ((a1 - a0) (reach^2 + (s - mid)^2)).
    let share = (a1 - a0) / (reach * f32(SPOT_TAPS));
    // The cluster's light cap and selection, as the lit pass's: a sample in
    // the cluster of the one before keeps its answer.
    var cluster = 0xffffffffu;
    var present = false;
    for (var t = 0u; t < SPOT_TAPS; t++) {
      let along = mid + reach * tan(mix(a0, a1, (f32(t) + jitter) / f32(SPOT_TAPS)));
      let p = frame.view_pos.xyz + dir * along;
      let here = cluster_at(uv * vol.extent.zw, p);
      if (here != cluster) {
        cluster = here;
        let word = cluster_word(here);
        let first = word & 0xffffffu;
        present = false;
        for (var k = 0u; k < (word >> 24u); k++) {
          if (cluster_ref(first + k) == i) { present = true; break; }
        }
      }
      if (!present) { continue; }
      let angles = mix(vec2<f32>(a0), vec2<f32>(a1), vec2<f32>(f32(t), f32(t + 1u)) / f32(SPOT_TAPS));
      var edges = clamp(vec2<f32>(mid) + reach * tan(angles), vec2<f32>(span.x), vec2<f32>(span.y));
      if (t == 0u) { edges.x = span.x; }
      if (t + 1u == SPOT_TAPS) { edges.y = span.y; }
      let m = vol_medium(p, dir, vec3<f32>(edges, edges.y - edges.x));
      if (m.sigma <= 0.0) { continue; }
      let off = along - mid;
      let trans = froxel_at(uv, along * forward).a;
      sum += trans * share * (reach * reach + off * off) * m.scatter * local_in(light, p, dir);
    }
  }
  return sum;
}

@fragment
fn fs_froxel(in: VolOut) -> @location(0) vec4<f32> {
  let depth = pixel_depth(in.pos.xy);
  let d = min(depth, vol.extent.x);
  let uv = in.pos.xy / vol.extent.zw;
  var o = froxel_at(uv, d);
  o = vec4<f32>(o.rgb + spots_in(uv, d), o.a);
  // The air past the grid, behind what the grid holds.
  let far = far_air(view_ray(uv), depth);
  o = vec4<f32>(o.rgb + o.a * far.rgb, o.a * far.a);
  // The water's fog has dimmed the scene already.
  return vec4<f32>(o.rgb, select(o.a, 1.0, VOL_WATER));
}

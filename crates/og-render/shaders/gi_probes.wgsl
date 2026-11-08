// The light volumes' buffer (lightmap.rs): the baked and realtime light
// volumes' header and probes, then the probe GI's header rows and probes
// (gi_common.wgsl). Read by every lit fragment through ambient_light and
// the probe GI's sampling (gi_sample.wgsl).
@group(0) @binding(10) var<storage, read> probe_data: ProbeWords;

// The probe GI's harmonics at a lit fragment's `p` for normal `n`: one
// gather, which the diffuse light (gi_ambient_from) and the reflections'
// occlusion (gi_reflected_from) both read.
// Whether a pipeline has the probe GI's gather (pipeline_features.rs).
override OG_GI: bool = true;

fn gi_sample_at(p: vec3<f32>, n: vec3<f32>) -> GiSample {
  if (!OG_GI) { return GiSample(vec4<f32>(0.0), vec4<f32>(0.0), vec4<f32>(0.0), 0.0); }
  if (probe_data.v[GI_HEADER_AT].x < 0.5) { return GiSample(vec4<f32>(0.0), vec4<f32>(0.0), vec4<f32>(0.0), 0.0); }
  let v = select(normalize(frame.view_pos.xyz - p), -frame.view_forward.xyz, frame.view_pos.w > 0.5);
  return gi_gather(p, n, v, true);
}

// `env` (an irradiance in the environment's units: the caller scales it by
// its intensity, frame.params.x) with the probe GI's `s` over it where the
// probes reach, along `n`: the light the scene sends the point, its sky
// occluded by what stands around it.
fn gi_ambient_from(env: vec3<f32>, s: GiSample, n: vec3<f32>) -> vec3<f32> {
  if (s.w <= 0.0) { return env; }
  return mix(env, gi_eval(s, n) / max(frame.params.x, 1e-4), s.w);
}

// gi_ambient_from with its own gather at `p`.
fn gi_ambient(env: vec3<f32>, n: vec3<f32>, p: vec3<f32>) -> vec3<f32> {
  return gi_ambient_from(env, gi_sample_at(p, n), n);
}

// `prefiltered` (the environment's reflection along `r` at `p`) occluded by
// what the probe GI's `s` sees along r: the probes' irradiance along r
// over the environment's, by luminance (1 where they see sky, towards 0 in
// a closed room), so a mirror under a roof stops reflecting the sky's
// full light. A reflection probe's capture already holds what stands
// around it, so where one covers `p` it is kept.
fn gi_reflected_from(prefiltered: vec3<f32>, r: vec3<f32>, p: vec3<f32>, s: GiSample) -> vec3<f32> {
  if (s.w <= 0.0) { return prefiltered; }
  let captured = select(0.0, probe_weight(p), frame.probe_min.w > 0.5);
  let luma = vec3<f32>(0.2126, 0.7152, 0.0722);
  let seen = dot(gi_eval(s, r), luma) / max(frame.params.x, 1e-4);
  let sky = max(dot(ambient_light_level(r, p), luma), 1e-4);
  let k = mix(1.0, clamp(seen / sky, 0.0, 1.0), s.w);
  return prefiltered * mix(k, 1.0, captured);
}

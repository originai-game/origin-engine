/// The most light volumes a frame samples.
pub const PROBE_VOLUMES: usize = 8;

/// The WGSL of the ambient-light hook over the probe buffer (frame binding
/// 10, declared with the frame group: gi_probes.wgsl), placed after
/// lighting.wgsl in the lit shaders.
pub fn probe_wgsl() -> String {
    format!("const PROBE_VOLUMES: u32 = {PROBE_VOLUMES}u;\n{AMBIENT_WGSL}")
}

const AMBIENT_WGSL: &str = r#"
// The baked lightmap texel of this fragment (fs_lightmap sets it; a: 1),
// irradiance over π.
var<private> baked_lightmap: vec4<f32>;

fn probe_irradiance(at: u32, n: vec3<f32>) -> vec3<f32> {
  let r = probe_data.v[at];
  let g = probe_data.v[at + 1u];
  let b = probe_data.v[at + 2u];
  return max(vec3<f32>(r.w + dot(r.xyz, n), g.w + dot(g.xyz, n), b.w + dot(b.xyz, n)), vec3<f32>(0.0));
}

// The light volumes' irradiance over π at p for normal n: the baked ones'
// (`baked`) and the realtime ones' (`live`, probe_grid.rs), each with how
// much of it to take (w: 1 inside a volume of its kind, falling to 0 over
// the volume's fade past its faces). A volume's header is three rows: its
// min and first probe, its max and packed counts (bit 21 on a realtime
// volume), its fade.
struct VolumeLight { baked: vec4<f32>, live: vec4<f32> };

fn light_volumes(p: vec3<f32>, n: vec3<f32>) -> VolumeLight {
  live_sh = array<vec4<f32>, 3>(vec4<f32>(0.0), vec4<f32>(0.0), vec4<f32>(0.0));
  var sum = array<vec3<f32>, 2>(vec3<f32>(0.0), vec3<f32>(0.0));
  var weight = array<f32, 2>(0.0, 0.0);
  for (var i = 0u; i < PROBE_VOLUMES; i++) {
    let h0 = probe_data.v[3u * i];
    let h1 = probe_data.v[3u * i + 1u];
    if (h0.w < 0.0) { break; }
    let packed = u32(h1.w);
    let kind = (packed >> 21u) & 1u;
    let c = vec3<u32>(packed & 127u, (packed >> 7u) & 127u, (packed >> 14u) & 127u);
    let cmax = vec3<f32>(c - vec3<u32>(1u));
    let size = max(h1.xyz - h0.xyz, vec3<f32>(1e-4));
    // Whole up to its faces, so the walls, floor and ceiling on them take
    // its probes, and fading out over its fade beyond (frame.wgsl).
    let w = region_weight(box_outside(p, h0.xyz, h1.xyz), probe_data.v[3u * i + 2u].x);
    if (w <= 0.0) { continue; }
    let g = clamp((p - h0.xyz) / size * cmax, vec3<f32>(0.0), cmax);
    let g0 = min(vec3<u32>(floor(g)), c - vec3<u32>(2u));
    let f = g - vec3<f32>(g0);
    var irr = vec3<f32>(0.0);
    for (var k = 0u; k < 8u; k++) {
      let o = vec3<u32>(k & 1u, (k >> 1u) & 1u, k >> 2u);
      let q = g0 + o;
      let wk = select(1.0 - f.x, f.x, o.x == 1u) * select(1.0 - f.y, f.y, o.y == 1u) * select(1.0 - f.z, f.z, o.z == 1u);
      let at = u32(h0.w) + ((q.z * c.y + q.y) * c.x + q.x) * 3u;
      irr += probe_irradiance(at, n) * wk;
      if (kind == 1u) {
        // The realtime probes' harmonics themselves, for their reflections.
        let ww = wk * w;
        live_sh[0] += probe_data.v[at] * ww;
        live_sh[1] += probe_data.v[at + 1u] * ww;
        live_sh[2] += probe_data.v[at + 2u] * ww;
      }
    }
    sum[kind] += irr * w;
    weight[kind] += w;
  }
  var out: VolumeLight;
  out.baked = select(vec4<f32>(0.0), vec4<f32>(sum[0] / weight[0], min(weight[0], 1.0)), weight[0] > 0.0);
  out.live = select(vec4<f32>(0.0), vec4<f32>(sum[1] / weight[1], min(weight[1], 1.0)), weight[1] > 0.0);
  if (weight[1] > 0.0) {
    let inv = 1.0 / weight[1];
    live_sh = array<vec4<f32>, 3>(live_sh[0] * inv, live_sh[1] * inv, live_sh[2] * inv);
  }
  return out;
}

// What `ambient_irradiance` leaves for `volume_reflected`: the realtime
// volumes' harmonics at the fragment and how much of them it took.
var<private> live_sh: array<vec4<f32>, 3>;
var<private> live_weight: f32;

// The realtime light volumes' reflection at `p` along `r` for
// `roughness`, given `prefiltered` (the reflected environment, or a
// ReflectionProbe's capture, in its units before its intensity): the
// light the probes' harmonics give along `r`, a soft image of the room
// (red off a red wall, light from a lit ceiling, dark below), which a
// sky's reflection would not be. Where a ReflectionProbe covers `p` its
// sharp capture (drawn with the probes' light) is kept for smooth
// surfaces, blurring into the probes' light as the roughness rises: a
// mirror in a room wants a probe. Outside every realtime volume (and on
// lightmapped fragments) `prefiltered` as it came.
fn volume_reflected(prefiltered: vec3<f32>, p: vec3<f32>, r: vec3<f32>, roughness: f32) -> vec3<f32> {
  if (live_weight <= 0.0) { return prefiltered; }
  let scale = 1.0 / max(frame.params.x, 1e-4);
  let along = max(vec3<f32>(live_sh[0].w + dot(live_sh[0].xyz, r), live_sh[1].w + dot(live_sh[1].xyz, r), live_sh[2].w + dot(live_sh[2].xyz, r)), vec3<f32>(0.0)) * scale;
  let probe = select(0.0, probe_weight(p), frame.probe_min.w > 0.5);
  let sharp = probe * (1.0 - smoothstep(0.4, 1.0, roughness));
  return mix(prefiltered, mix(along, prefiltered, sharp), live_weight);
}

// The diffuse ambient light of a lit fragment, in the environment
// irradiance's units (the caller scales it by the environment intensity,
// which baked and captured light do not take): its lightmap texel, else
// the realtime light volumes around it over the baked ones over `env`.
fn ambient_irradiance(env: vec3<f32>, p: vec3<f32>, n: vec3<f32>) -> vec3<f32> {
  let scale = 1.0 / max(frame.params.x, 1e-4);
  if (baked_lightmap.a > 0.5) { return baked_lightmap.rgb * scale; }
  let v = light_volumes(p, n);
  live_weight = v.live.w;
  let baked = mix(env, v.baked.rgb * scale, v.baked.w);
  return mix(baked, v.live.rgb * scale, v.live.w);
}
"#;

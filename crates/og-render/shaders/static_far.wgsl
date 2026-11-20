// The static lights' far field (static_far.rs): a grid of irradiance
// probes round the camera holding the diffuse light of the static lights
// past their near field, which the lit shaders no longer shade per pixel
// (frame.wgsl `static_near`). In the light volumes' buffer (frame binding
// 10, gi_probes.wgsl) after the probe GI's region; its header rows
// FAR_HEADER_AT.. are the first probe's position and first vec4 (w -1: no
// field), then the spacing. Each probe is the irradiance over π of red,
// green and blue as `[c1.x, c1.y, c1.z, c0]`, x fastest.

const FAR_HEADER_AT: u32 = 33u;
const FAR_COUNTS: vec3<u32> = vec3<u32>(32u, 12u, 32u);

// The far lights' irradiance over π at p along n: each of the eight
// probes round p evaluated along n, weighed trilinearly, fading out over
// the grid's outer cell (zero outside it). One vec3 is carried through
// the gather, so the lit shaders keep their registers.
fn static_far_irradiance(p: vec3<f32>, n: vec3<f32>) -> vec3<f32> {
  let h0 = probe_data.v[FAR_HEADER_AT];
  let spacing = probe_data.v[FAR_HEADER_AT + 1u].x;
  let cmax = vec3<f32>(FAR_COUNTS - vec3<u32>(1u));
  let g = (p - h0.xyz) / spacing;
  let edge = min(g, cmax - g);
  let w = clamp(min(edge.x, min(edge.y, edge.z)), 0.0, 1.0);
  if (w <= 0.0 || h0.w < 0.0) { return vec3<f32>(0.0); }
  let g0 = min(vec3<u32>(floor(g)), FAR_COUNTS - vec3<u32>(2u));
  let f = g - vec3<f32>(g0);
  var irr = vec3<f32>(0.0);
  for (var k = 0u; k < 8u; k++) {
    let c = vec3<u32>(k & 1u, (k >> 1u) & 1u, k >> 2u);
    let q = g0 + c;
    let wk = select(1.0 - f.x, f.x, c.x == 1u) * select(1.0 - f.y, f.y, c.y == 1u) * select(1.0 - f.z, f.z, c.z == 1u);
    let at = u32(h0.w) + ((q.z * FAR_COUNTS.y + q.y) * FAR_COUNTS.x + q.x) * 3u;
    let r = probe_data.v[at];
    let gr = probe_data.v[at + 1u];
    let b = probe_data.v[at + 2u];
    irr += max(vec3<f32>(r.w + dot(r.xyz, n), gr.w + dot(gr.xyz, n), b.w + dot(b.xyz, n)), vec3<f32>(0.0)) * wk;
  }
  return irr * w;
}

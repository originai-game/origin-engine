// Screen-space global illumination (ssgi.rs), after fxpost.wgsl: in the
// AO pass's place, each pixel gathers the light the screen shows around
// it over horizon slices, as GTAO gathers its visibility (fxpost.wgsl
// gtao_bent), after Therrien, Levesque and Gilet, "Screen Space Indirect
// Lighting with Visibility Bitmask" (2023). (gi1.x + 1) / 2 slices pass
// through the view ray, each walked both ways through the prepass's view
// depth, gi0.w taps a side. A slice's half circle about the normal is 32
// sectors of equal cosine weight: each tap's surface covers the sectors
// it spans, from where it meets the tap before it to where it meets the
// next and `thickness` behind, and the sectors nothing nearer the pixel
// covered take the light that surface sent last frame, read from tex_a
// (TAA's history, or the GI's copy of the lit image) where the prepass
// velocity says the tap's pixel was. So a pixel reads light only where a
// surface comes into its view, a few times a slice, and no direction is
// counted twice. Writes r: the ambient occlusion (the scene's AO term, or
// the covered sectors' share), gba: the sectors' light (the irradiance
// over pi a diffuse surface receives), which fs_gi_blur and
// fs_gi_upsample then filter as the AO's blur does.

// With the probe GI on as well (`method: both`), the gather reaches the
// near field, one probe spacing, and the sectors it lights replace the
// probes' light from their directions and no more: the pass gathers the
// probes once (gi_sample.wgsl) and takes from their harmonics the light
// along the lit sectors' directions, which a lit pass's ambient occlusion
// alone would take as an even share of every direction's. Sectors whose
// light the screen cannot give leave it to the probes.

// The light volumes' buffer with the probe GI's probes (the frame group's
// binding 10, gi_probes.wgsl).
@group(0) @binding(7) var<storage, read> probe_data: ProbeWords;

// Sectors of visibility in a slice: one u32's bits.
const GI_SECTORS: f32 = 32.0;
// How much farther or nearer than the tap before it a tap may lie, per
// world unit between them across the view, and still be the same surface
// (a floor seen 14° from grazing).
const GI_SLOPE: f32 = 4.0;
// The sine of the most three taps in a row may bend and still lie on one
// plane, however steep.
const GI_BEND: f32 = 0.05;
// How far a surface that leaves the screen is taken to go on beyond it, in
// lengths of the straight run it left on: none loses the light of a floor
// or wall that runs on under the frame, a whole one gathers more than the
// frame's surfaces send (tests/gi.rs's ceiling).
const GI_BEYOND: f32 = 0.5;

// acos(x), within 7e-5 rad (Abramowitz and Stegun 4.4.45).
fn gi_acos(x: f32) -> f32 {
  let a = abs(x);
  let r = sqrt(max(1.0 - a, 0.0)) * (1.5707288 + a * (-0.2121144 + a * (0.0742610 - a * 0.0187293)));
  return select(PI - r, r, x >= 0.0);
}

// A slice's half circle about the pixel's normal, as the cosine about that
// normal weighs it: `c`, `s` the normal's angle from the view ray in the
// slice (its cosine and sine), `lo` the weight below the view ray and
// `inv_span` one over the whole.
struct GiArc {
  c: f32,
  s: f32,
  lo: f32,
  inv_span: f32,
}

// Where the direction to `at` in a slice (x along the slice, y towards
// the eye, from the pixel) falls in its half circle about the normal, 0 at the horizon
// behind the slice's direction and 1 at the one ahead, in equal shares of
// the light a diffuse surface takes: GTAO's arc integral (fxpost.wgsl
// gtao_bent's arc0 and arc1, Jimenez et al. 2016 eq. 7) from the view ray
// to it. Outside the half circle, `end`: the horizon on the tap's side
// of the pixel (the half circle holds the view ray, so that is the nearer
// one; not the point's own side, which the pixel's lift off its surface turns
// over for the taps beside it).
fn gi_sector(at: vec2<f32>, arc: GiArc, end: f32) -> f32 {
  if (at.x * arc.s + at.y * arc.c <= 0.0) { return end; }
  let w = normalize(at);
  let side = select(-1.0, 1.0, w.x >= 0.0);
  let theta = side * gi_acos(w.y);
  let g = side * 0.5 * (arc.c * w.x * w.x + arc.s * (theta - w.x * w.y));
  return clamp((g - arc.lo) * arc.inv_span, 0.0, 1.0);
}

// Where the points along one side of a slice lie in it, from the pixel
// (x along the slice, y towards the eye): the projection's inverse written
// out (view_pos's two matrix products a tap were a third of the pass) and
// taken along the slice's line on the screen, where it is linear in how
// far out a point is. `at` + `step` × pixels out, times the depth under a
// perspective camera (`scale` 1) or as it is under an orthographic one
// (`scale` 0), plus `depth` × the depth, less `origin`.
struct GiLine {
  at: vec2<f32>,
  step: vec2<f32>,
  depth: vec2<f32>,
  origin: vec2<f32>,
  scale: f32,
}

// The point `off` pixels out along the line with view depth `depth`.
fn gi_in_slice(line: GiLine, off: f32, depth: f32) -> vec2<f32> {
  return (line.at + line.step * off) * mix(1.0, depth, line.scale) + line.depth * depth - line.origin;
}

// The sectors from `lo` to `hi` (0..1) as bits, each end rounded down
// after `dither` (0..1): over the pixels of the noise pattern a sector
// part covered is set that part of the time.
fn gi_bits(lo: f32, hi: f32, dither: f32) -> u32 {
  let a = u32(clamp(lo * GI_SECTORS + dither, 0.0, GI_SECTORS));
  let b = u32(clamp(hi * GI_SECTORS + dither, 0.0, GI_SECTORS));
  let below_a = select((1u << a) - 1u, 0xffffffffu, a >= 32u);
  let below_b = select((1u << b) - 1u, 0xffffffffu, b >= 32u);
  return below_b & ~below_a;
}

// The pixel's noise: x turns the slices, y offsets the taps along them,
// z dithers the sectors. The interleaved 3x3 pattern (fxpost.wgsl ao_noise)
// gives each of the blur's nine pixels its own of each; under TAA the
// frame (gi1.y) moves all three along the R2 sequence.
fn gi_noise(pos: vec2<f32>) -> vec3<f32> {
  let cell = floor(ao_noise(pos) * 9.0);
  let at = vec3<i32>(vec2<i32>(pos), i32(fxp.gi1.y));
  let white = vec3<f32>(lattice3(at, 1u), lattice3(at, 2u), lattice3(at, 3u));
  return fract((vec3<f32>(cell, cell * 4.0, 0.0) + white * vec3<f32>(1.0, 1.0, 9.0)) / 9.0 + fxp.gi1.y * vec3<f32>(0.7548776662, 0.5698402910, 0.0));
}

@fragment
fn fs_ssgi(in: VsOut) -> @location(0) vec4<f32> {
  let px = ao_pixel(in.pos.xy);
  let d = depth_at(px);
  // The AO debug views (DebugView::AoDepth) as the AO pass writes them.
  if (fxp.cam.w > 0.5 && fxp.cam.w < 1.5) { return vec4<f32>(fract(d), 0.0, 0.0, 0.0); }
  if (is_sky(d)) { return vec4<f32>(1.0, 0.0, 0.0, 0.0); }
  let uv = ao_uv(px, in.uv);
  let n = normalize((fxp.view * vec4<f32>(gbuffer_normal(px), 0.0)).xyz);
  let ortho_cam = fxp.cam.y > 0.5;
  // Lifted off the surface, so its own plane's taps lie under the horizon.
  let p = view_pos(uv, d) + n * (0.01 + 0.002 * d);
  let v = select(normalize(-p), vec3<f32>(0.0, 0.0, 1.0), ortho_cam);
  // The pixel's distance from the eye along its view ray.
  let p_len = length(p);
  let probes = probe_data.v[GI_HEADER_AT].x > 0.5;
  // With the probes, the gather reaches one spacing of their finest
  // cascade: the near field, which the probes cannot resolve. Farther,
  // the probes see what the screen would, and L1 harmonics are too smooth
  // to say how much of their light came from the directions a far surface
  // fills, so the two would count it twice.
  let reach = select(fxp.gi0.x, min(fxp.gi0.x, probe_data.v[GI_HEADER_AT + 1u].w), probes);
  let thickness = fxp.gi0.y;
  let steps = max(u32(fxp.gi0.w), 1u);
  let taps = steps + 1u;
  let slices = max((u32(fxp.gi1.x) + 1u) / 2u, 1u);
  let known = fxp.gi1.z > 0.5;
  let radius_px = reach * pixels_per_unit(d);
  let noise = gi_noise(in.pos.xy);
  // A uv's view-space point over its depth (perspective) or as it is
  // (orthographic): ndc × `unproject` + `centre`.
  let unproject = 1.0 / vec2<f32>(fxp.proj[0][0], fxp.proj[1][1]);
  let centre = select(vec2<f32>(fxp.proj[2][0], fxp.proj[2][1]), -vec2<f32>(fxp.proj[3][0], fxp.proj[3][1]), ortho_cam) * unproject;
  let ndc = vec2<f32>(uv.x * 2.0 - 1.0, 1.0 - uv.y * 2.0);
  // World units a pixel spans at depth 1 (perspective) or at any depth.
  let unit_px = 1.0 / (fxp.proj[1][1] * 0.5 * fxp.screen.y);
  let dsize = vec2<i32>(textureDimensions(tex_depth));
  let top = i32(textureNumLevels(tex_depth)) - 1;
  let gsize = vec2<i32>(textureDimensions(tex_gbuffer));
  let pos = vec2<f32>(px) + 0.5;
  var light = vec3<f32>(0.0);
  var hits = 0.0;
  // The lit sectors' directions, each by its weight (view space).
  var towards = vec3<f32>(0.0);
  for (var s = 0u; s < slices; s++) {
    // The slice's direction about the view ray, and along the screen (as
    // gtao_bent's).
    let phi = (f32(s) + noise.x) * PI / f32(slices);
    let dir = gtao_rotate(vec3<f32>(0.0, 0.0, 1.0), v, vec3<f32>(cos(phi), sin(phi), 0.0));
    let along = select(-p.z * dir.xy + p.xy * dir.z, dir.xy, ortho_cam);
    let omega = normalize(vec2<f32>(along.x, -along.y));
    let axis = cross(dir, v);
    let proj_n = n - axis * dot(n, axis);
    let proj_len = length(proj_n);
    var arc: GiArc;
    arc.c = clamp(dot(proj_n, v) / max(proj_len, 1e-5), 0.0, 1.0);
    arc.s = select(-1.0, 1.0, dot(proj_n, dir) >= 0.0) * sqrt(1.0 - arc.c * arc.c);
    let gamma = select(-1.0, 1.0, arc.s >= 0.0) * gi_acos(arc.c);
    arc.lo = -0.5 * (arc.c + arc.s * (gamma - 0.5 * PI));
    arc.inv_span = 1.0 / (arc.c + gamma * arc.s);
    // One sector's share of the light the pixel gathers.
    let per_bit = proj_len * (arc.c + gamma * arc.s) / (GI_SECTORS * f32(slices));
    var mask = 0u;
    for (var side = 0u; side < 2u; side++) {
      let om = omega * select(-1.0, 1.0, side == 1u);
      // The pixel's plane is this side's horizon.
      let end = f32(side);
      // As far as the reach, or the screen's edge.
      let room = select(pos, fxp.screen.xy - pos, om > vec2<f32>(0.0)) / max(abs(om), vec2<f32>(1e-5));
      let side_px = min(radius_px, min(room.x, room.y));
      let to_edge = side_px < radius_px;
      let jitter = fract(noise.y + 0.5 * f32(side));
      // The side's line (GiLine): a pixel along `om` moves the ndc by
      // (2 om.x / w, -2 om.y / h).
      let ray = vec3<f32>(ndc * unproject + centre, select(-1.0, 0.0, ortho_cam));
      let ray_px = vec3<f32>(vec2<f32>(2.0 * om.x, -2.0 * om.y) * fxp.screen.zw * unproject, 0.0);
      var line: GiLine;
      line.at = vec2<f32>(dot(ray, dir), dot(ray, v));
      line.step = vec2<f32>(dot(ray_px, dir), dot(ray_px, v));
      line.depth = select(vec2<f32>(0.0), vec2<f32>(-dir.z, -v.z), ortho_cam);
      line.origin = vec2<f32>(dot(p, dir), dot(p, v));
      line.scale = select(1.0, 0.0, ortho_cam);
      // The tap before the one being placed: whether it is a surface in
      // reach, where it is in the slice, its depth and how far out.
      var a_ok = false;
      var a_w = vec2<f32>(0.0);
      var a_d = d;
      var a_off = 0.0;
      // The one being placed: at first the pixel itself, a surface that
      // takes no sectors (`c_place`).
      var c_place = false;
      var c_ok = true;
      var c_w = vec2<f32>(0.0);
      var c_d = d;
      var c_off = 0.0;
      var c_uv = uv;
      // Whether the two are one surface, and where it passes from the
      // one to the other.
      var linked = false;
      var u_shared = end;
      // Where the straight run of taps the one being placed is on began.
      var run_w = vec2<f32>(0.0);
      var on_screen = true;
      for (var k = 0u; k < taps; k++) {
        // The next tap out, read before the one it follows is placed.
        var n_read = false;
        var n_ok = false;
        var n_w = vec2<f32>(0.0);
        var n_d = SKY_VIEW_DEPTH;
        var n_off = c_off;
        var n_uv = uv;
        if (k < steps && on_screen) {
          let t = (f32(k) + jitter) / f32(steps);
          // Finer near the pixel for the contact, and at least 1.3
          // pixels out (gtao_bent), on a pixel's centre.
          n_off = t * t * side_px + 1.3;
          // The last one on the screen's edge, where the side ends there:
          // what leaves the screen is seen leaving it.
          if (k + 1u == steps && to_edge) { n_off = side_px - 1.0; }
          let n_px = round(om * n_off);
          n_uv = uv + n_px * fxp.screen.zw;
          if (any(n_uv < vec2<f32>(0.0)) || any(n_uv > vec2<f32>(1.0))) {
            on_screen = false;
          } else {
            n_read = true;
            n_d = depth_mip_tap(n_uv, dsize, depth_tap_level(n_off, top));
            if (!is_sky(n_d)) {
              // Where the pixel it reads lies along the line.
              n_w = gi_in_slice(line, dot(n_px, om), n_d);
              n_ok = dot(n_w, n_w) <= reach * reach;
            }
          }
        }
        if (!c_place && !n_read) { break; }
        // The tap stands for the surface from where it meets the tap
        // before it to where it meets the next. One surface with a
        // neighbour (the depth buffer between the two unbroken, so it
        // shows a point in every direction between theirs), they meet
        // midway. Across an edge, the nearer one's surface ends
        // somewhere between them, halfway at its own depth, and the
        // farther one's goes on behind it. Beside the sky, halfway.
        var u_in = u_shared;
        let from_a = linked;
        if (c_place && !linked) {
          let inner = select(0.5 * (a_off + c_off), a_off, a_ok && c_d > a_d);
          u_in = gi_sector(gi_in_slice(line, inner, c_d), arc, end);
        }
        // One surface: three taps in a row on one line, however steep,
        // or no farther apart in depth than the thickness and what a
        // slope gives over the distance between them.
        let ac = c_w - a_w;
        let cn = n_w - c_w;
        let bend = ac.x * cn.y - ac.y * cn.x;
        let straight = a_ok && c_ok && n_ok && bend * bend <= GI_BEND * GI_BEND * dot(ac, ac) * dot(cn, cn);
        linked = c_ok && n_ok && (straight
          || abs(n_d - c_d) <= thickness + GI_SLOPE * (n_off - c_off) * unit_px * select(min(n_d, c_d), 1.0, ortho_cam));
        if (c_place) {
          var u_far = u_in;
          if (linked) {
            u_shared = gi_sector(0.5 * (c_w + n_w), arc, end);
          } else {
            var outer = c_off + 0.5 * (c_off - a_off);
            if (n_read) { outer = select(0.5 * (c_off + n_off), n_off, n_ok && c_d > n_d); }
            u_shared = gi_sector(gi_in_slice(line, outer, c_d), arc, end);
            // A surface that leaves the screen goes on beyond it
            // (GI_BEYOND), within the reach.
            if (!n_read && from_a && to_edge) {
              let run = c_w - run_w;
              let more = min(GI_BEYOND, max(reach - length(c_w), 0.0) / max(length(run), 1e-5));
              u_far = gi_sector(c_w + run * more, arc, end);
            }
          }
          // And `thickness` behind the depth buffer, along the tap's
          // view ray.
          let away = select(normalize(vec2<f32>(c_w.x, c_w.y - p_len)), vec2<f32>(0.0, -1.0), ortho_cam);
          let u_back = gi_sector(c_w + away * thickness, arc, end);
          let lo = min(min(u_in, u_shared), min(u_back, u_far));
          let hi = max(max(u_in, u_shared), max(u_back, u_far));
          // The sectors nothing nearer the pixel filled.
          let bits = gi_bits(lo, hi, noise.z) & ~mask;
          mask |= bits;
          if (bits != 0u) {
            let dist = length(c_w);
            let fade = 1.0 - smoothstep(0.6, 1.0, dist / reach);
            let share = f32(countOneBits(bits)) * per_bit * fade;
            // Without last frame's light, facing away or where the
            // surface lay off screen then, the probes give those
            // directions' light; without them they are occluded.
            if (!probes) { hits += share; }
            if (known) {
              let g = textureLoad(tex_gbuffer, prepass_px(vec2<i32>(c_uv * fxp.screen.xy), gsize), 0);
              let f = dir * c_w.x + v * c_w.y;
              // Only the side of a surface that faces the pixel sends it
              // light.
              let hit_n = (fxp.view * vec4<f32>(oct_decode(g.xy), 0.0)).xyz;
              // Where the tap's surface was on the previous frame
              // (gbuffer zw: its screen-space velocity, uv per frame).
              let prev = c_uv - g.zw;
              if (dot(hit_n, f) <= 0.1 * dist && all(prev >= vec2<f32>(0.0)) && all(prev <= vec2<f32>(1.0))) {
                var l = max(textureSampleLevel(tex_a, fx_linear, prev, 0.0).rgb, vec3<f32>(0.0));
                // Fireflies: a tiny bright emitter a few taps meet is
                // held to a level the blur can spread.
                l *= min(1.0, 32.0 / max(luma(l), 1e-4));
                // With the probes, a surface fading out by the screen's
                // edge gives way to them.
                let edge = min(min(prev.x, 1.0 - prev.x), min(prev.y, 1.0 - prev.y));
                let seen = share * select(1.0, smoothstep(0.0, 0.05, edge), probes);
                light += l * seen;
                if (probes) {
                  hits += seen;
                  towards += f * (seen / max(dist, 1e-5));
                }
              }
            }
          }
        }
        if (!straight) { run_w = c_w; }
        a_ok = c_ok;
        a_w = c_w;
        a_d = c_d;
        a_off = c_off;
        c_place = n_ok;
        c_ok = n_ok;
        c_w = n_w;
        c_d = n_d;
        c_off = n_off;
        c_uv = n_uv;
      }
    }
  }
  light *= fxp.gi0.z;
  var ao = 1.0 - fxp.gi2.x * hits;
  if (fxp.gi1.w > 1.5) {
    ao = ssao_term(in);
  } else if (fxp.gi1.w > 0.5) {
    ao = gtao_term(in);
  }
  if (probes) {
    // The lit pass adds the probes' light times `ao`; what reaches the
    // pixel is theirs less what the lit sectors replace. With E/π along a
    // unit n c0 + c1·n (gi_sample.wgsl), the radiance they stand for is
    // L1, L(d) = c0 + 1.5 c1·d, and the sectors share the hemisphere by
    // cosine, so the lit ones' share of it over π is h c0 + 1.5 c1·s,
    // with h their weight and s their weighted directions. The bounce
    // carries the difference: exact for L1 light, where an even share
    // (`ao` alone) takes too little from a bright wall beside a dark sky
    // and counts its light twice.
    let to_world = transpose(mat3x3<f32>(fxp.view[0].xyz, fxp.view[1].xyz, fxp.view[2].xyz));
    let wn = to_world * n;
    let wp = to_world * (p - fxp.view[3].xyz);
    // Towards the eye, or under an orthographic camera its axis (as the
    // lit pass's gi_sample_at).
    let eye = -(to_world * fxp.view[3].xyz);
    let towards_eye = select(normalize(eye - wp), to_world * vec3<f32>(0.0, 0.0, 1.0), ortho_cam);
    let g = gi_gather(wp, wn, towards_eye, true);
    if (g.w > 0.0) {
      let s = to_world * towards;
      let all = gi_eval(g, wn);
      let hit = vec3<f32>(g.r.w, g.g.w, g.b.w) * hits + 1.5 * vec3<f32>(dot(g.r.xyz, s), dot(g.g.xyz, s), dot(g.b.xyz, s));
      light += g.w * (max(all - hit, vec3<f32>(0.0)) - ao * all);
    }
  }
  return vec4<f32>(ao, max(light, vec3<f32>(0.0)));
}

// Symmetric 3x3 bilateral blur over the GI's four channels, weighted by depth
// and normal alignment (eliminates the half-pixel shift of an asymmetric 4x4).
@fragment
fn fs_gi_blur(in: VsOut) -> @location(0) vec4<f32> {
  let px = vec2<i32>(in.pos.xy);
  if (fxp.cam.w > 0.5) { return textureLoad(tex_b, px, 0); }
  let k = ao_divisor();
  let d0 = depth_at(px * k);
  if (ao_blur_sky(d0)) { return vec4<f32>(1.0, 0.0, 0.0, 0.0); }
  let n0 = gbuffer_normal(px * k);
  let size = vec2<i32>(textureDimensions(tex_b));
  let scale = max(d0 * 0.04, 1e-3);
  var sum = vec4<f32>(0.0);
  var weight = 0.0;
  for (var y = -1; y <= 1; y++) {
    for (var x = -1; x <= 1; x++) {
      let q = clamp(px + vec2<i32>(x, y), vec2<i32>(0), size - 1);
      let facing = max(dot(gbuffer_normal(q * k), n0), 0.0);
      let w = exp(-abs(depth_at(q * k) - d0) / scale) * facing * facing * facing * facing + 1e-6;
      sum += textureLoad(tex_b, q, 0) * w;
      weight += w;
    }
  }
  // GTAO's visibility (r) overshoots 1 before it is averaged (fxpost.wgsl
  // gtao_bent); after, it is clamped. Keep `.gba` signed: `fs_gtao_bent`
  // stores world-space bent normals in [-1, 1], while SSGI bounce inputs are
  // already non-negative and stay non-negative under positive weights.
  let mean = sum / weight;
  return vec4<f32>(min(mean.r, 1.0), mean.gba);
}

// Normal- and depth-aware bilateral upsample over the GI's four channels.
@fragment
fn fs_gi_upsample(in: VsOut) -> @location(0) vec4<f32> {
  let px = vec2<i32>(in.pos.xy);
  let d0 = depth_at(px);
  if (ao_blur_sky(d0)) { return vec4<f32>(1.0, 0.0, 0.0, 0.0); }
  let n0 = gbuffer_normal(px);
  let size = vec2<i32>(textureDimensions(tex_b));
  let base = px / 2;
  let f = vec2<f32>(px - base * 2) * 0.5;
  let scale = max(d0 * 0.04, 1e-3);
  var sum = vec4<f32>(0.0);
  var weight = 0.0;
  var nearest = vec4<f32>(1.0, 0.0, 0.0, 0.0);
  var nearest_dz = 3.4e38;
  for (var y = 0; y <= 1; y++) {
    for (var x = 0; x <= 1; x++) {
      let q = min(base + vec2<i32>(x, y), size - 1);
      let a = textureLoad(tex_b, q, 0);
      let dz = abs(depth_at(q * 2) - d0);
      let facing = max(dot(gbuffer_normal(q * 2), n0), 0.0);
      let bilinear = select(1.0 - f.x, f.x, x == 1) * select(1.0 - f.y, f.y, y == 1);
      let w = bilinear * exp(-dz / scale) * (facing * facing + 1e-4);
      sum += a * w;
      weight += w;
      if (dz < nearest_dz) {
        nearest_dz = dz;
        nearest = a;
      }
    }
  }
  return select(nearest, sum / max(weight, 1e-5), weight > 1e-3);
}

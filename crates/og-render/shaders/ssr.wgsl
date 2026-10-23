// Screen-space reflections (ssr.rs; the full tier only), after fxpost.wgsl:
// each glossy pixel marches its reflected ray in view space through the
// prepass's view depth and, where the ray passes behind a surface by less
// than the thickness, blends in the lit image there, weighted by Fresnel
// and gloss and faded at the screen's edges, with distance and for rays
// turning towards the camera. tex_a: the lit image.

// The prepass's reflectivity (tex_depth.g): x the gloss (1 - roughness),
// y the metalness, packed in 255ths with the metalness times 256 above.
fn ssr_surface(px: vec2<i32>) -> vec2<f32> {
  let size = vec2<i32>(textureDimensions(tex_depth));
  let g = textureLoad(tex_depth, clamp(px, vec2<i32>(0), size - 1), 0).g;
  let metal = floor(g / 256.0);
  return vec2<f32>((g - metal * 256.0) / 255.0, metal / 255.0);
}

// A view-space point's screen uv.
fn ssr_uv(q: vec3<f32>) -> vec2<f32> {
  let c = fxp.proj * vec4<f32>(q, 1.0);
  let ndc = c.xy / c.w;
  return vec2<f32>(ndc.x * 0.5 + 0.5, 0.5 - ndc.y * 0.5);
}

@fragment
fn fs_ssr(in: VsOut) -> @location(0) vec4<f32> {
  let px = vec2<i32>(in.pos.xy);
  let lit = textureLoad(tex_a, px, 0);
  let d = depth_at(px);
  let surface = ssr_surface(px);
  let gloss = surface.x;
  if (is_sky(d) || gloss < 0.25 || fxp.cam.y > 0.5 || fxp.ssr.w <= 0.0) { return lit; }
  let uv = (vec2<f32>(px) + 0.5) * fxp.screen.zw;
  let p = view_pos(uv, d);
  let n = normalize((fxp.view * vec4<f32>(world_normal(px), 0.0)).xyz);
  let v = normalize(p);
  let r = normalize(reflect(v, n));
  // Rays back towards the camera (+z in view space) leave through what the
  // screen cannot show.
  let facing = 1.0 - smoothstep(0.0, 0.5, r.z);
  if (facing <= 0.0) { return lit; }
  let steps = max(u32(fxp.ssr2.x), 1u);
  let max_distance = fxp.ssr.x;
  let thickness = fxp.ssr.y;
  // Steps grow along the ray (fine near the surface, coarse far away),
  // started at a per-pixel offset so the banding becomes noise.
  let jitter = ign(in.pos.xy);
  let origin = p + n * (0.01 + 0.002 * d);
  var t_prev = 0.0;
  var hit = false;
  var t_hit = 0.0;
  for (var i = 0u; i < steps; i++) {
    let k = (f32(i) + jitter) / f32(steps);
    let t = k * k * max_distance;
    let q = origin + r * t;
    if (q.z > -fxp.cam.z) { break; }
    let s = ssr_uv(q);
    if (any(s < vec2<f32>(0.0)) || any(s > vec2<f32>(1.0))) { break; }
    let behind = -q.z - depth_uv(s);
    if (behind > 0.0) {
      if (behind < thickness + (t - t_prev)) {
        // Bisect between the last step in front and this one.
        var a = t_prev;
        var b = t;
        for (var j = 0; j < 5; j++) {
          let m = 0.5 * (a + b);
          let qm = origin + r * m;
          if (-qm.z - depth_uv(ssr_uv(qm)) > 0.0) { b = m; } else { a = m; }
        }
        t_hit = b;
        hit = true;
      }
      break;
    }
    t_prev = t;
  }
  if (!hit) { return lit; }
  let hit_uv = ssr_uv(origin + r * t_hit);
  if (is_sky(depth_uv(hit_uv))) { return lit; }
  let edge = min(min(hit_uv.x, 1.0 - hit_uv.x), min(hit_uv.y, 1.0 - hit_uv.y));
  let fade = smoothstep(0.0, max(fxp.ssr.z, 1e-3), edge)
    * (1.0 - smoothstep(0.7, 1.0, t_hit / max_distance))
    * facing;
  let reflected = textureSampleLevel(tex_a, fx_linear, hit_uv, 0.0).rgb;
  let n_dot_v = clamp(dot(n, -v), 0.0, 1.0);
  let f0 = mix(0.04, 1.0, surface.y);
  let fresnel = f0 + (1.0 - f0) * pow(1.0 - n_dot_v, 5.0);
  // A metal tints what it reflects with its own colour (the lit pixel's hue).
  let tint = mix(vec3<f32>(1.0), lit.rgb / max(max(lit.r, max(lit.g, lit.b)), 1e-4), surface.y);
  let w = clamp(fresnel * gloss * gloss * fade * fxp.ssr.w, 0.0, 1.0);
  return vec4<f32>(mix(lit.rgb, reflected * tint, w), lit.a);
}

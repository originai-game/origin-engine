// The sea of water.wgsl (appended to it): its waves and their slopes, its surf and foam, its vertices and the
// probe of its heights.

// How much of a wave of wavenumber k samples `cell` metres apart hold: all of one eight cells long, none under four
// (water/sea.rs sea_keep).
fn sea_keep(k: f32, cell: f32) -> f32 {
  return 1.0 - smoothstep(0.785, 1.571, k * cell);
}

// sea_keep of the wave of vector k (rad/m) for a pixel that moves gx and gy metres over the surface across and down
// the screen: by the phase the wave turns across it, so the filter is the pixel's own shape (stretched tens of times
// along the view near the horizon), not a circle about its longest side.
fn pixel_keep(k: vec2<f32>, gx: vec2<f32>, gy: vec2<f32>) -> f32 {
  return 1.0 - smoothstep(0.785, 1.571, max(abs(dot(k, gx)), abs(dot(k, gy))));
}

// The FFT cascade c's uv at rest point p (frame xz).
fn cascade_uv(c: u32, p: vec2<f32>) -> vec2<f32> {
  let t = water.cascades[c];
  return p * t.x + t.yz;
}

// The seabed's rows and the sea's waves for shore.wgsl (prepended).
fn sea_shore(i: u32) -> vec4<f32> {
  return water.shore[i];
}

fn sea_wave(i: u32) -> vec4<f32> {
  return water.swell[i];
}

// The sea's displacement (x, height, z) at rest point p (frame xz) as vertices `cell` metres apart draw it: every wave
// of og_core::ocean, the function the tick floats bodies on (og.physics.waterHeight), each left out only where the
// vertices are too far apart to hold it and shoaled by the seabed under p; then the FFT's waves, shorter than the
// shortest of those, read at the mip whose texels are the cells.
fn ocean_displace(p: vec2<f32>, cell: f32) -> vec3<f32> {
  let sh = shoaling(p);
  let d = ocean_swell(p, cell, sh) + ocean_cascades(p, cell, sh);
  // W3: the hulls' wakes on top, where the waves carry the point.
  return d + vec3<f32>(0.0, water_wake(p + d.xz).x, 0.0);
}

// ocean_displace's waves of og_core::ocean at rest point p (Ocean::displacement).
fn ocean_swell(p: vec2<f32>, cell: f32, sh: Shoal) -> vec3<f32> {
  var d = vec3<f32>(0.0);
  for (var i = 0u; i < 32u; i++) {
    let a = water.swell[2u * i];
    let b = water.swell[2u * i + 1u];
    let k = length(a.xy);
    let keep = sea_keep(k, cell) * shoal_keep(k, sh);
    let th = dot(a.xy, p) + a.w;
    d += vec3<f32>(b.x * cos(th), a.z * sin(th), b.y * cos(th)) * keep;
  }
  if (sh.amp > 0.0) {
    let keep = sea_keep(water.shore[1].z, cell);
    d += vec3<f32>(sh.inshore.x * sh.lean * cos(sh.phase), sh.amp * sin(sh.phase),
      sh.inshore.y * sh.lean * cos(sh.phase)) * keep;
  }
  return d;
}

// ocean_displace's FFT waves (render only) at rest point p, each cascade faded once its longest waves are under four
// cells.
fn ocean_cascades(p: vec2<f32>, cell: f32, sh: Shoal) -> vec3<f32> {
  var d = vec3<f32>(0.0);
  for (var c = 0u; c < u32(water.fft.x); c++) {
    let lod = log2(max(cell * 256.0 / water.cascades[c].w, 1.0));
    let fade = (1.0 - smoothstep(4.0, 6.0, lod)) * shoal_keep(water.bands[c], sh);
    if (fade <= 0.0) { continue; }
    d += textureSampleLevel(heights, ripple_sampler, cascade_uv(c, p), 3u * c, min(lod, 4.0)).xyz * fade;
  }
  return d;
}

// The sea's normal at rest point p (frame xz), each wave faded by the phase it turns across the pixel (gx, gy: a wave
// whose crests run along the view is held to the horizon, one across it only near; pixel_keep) and the cascades
// filtered over the same pixel; the Jacobian of its horizontal displacement (`jac`: under 1
// where the crests bunch, under 0 where they fold); and as `cov`, the covariance of the slopes the pixel holds
// unresolved (xx, zz, xz): the faded waves' ((k a)² / 2 along each one's direction), the cascades' within the pixel
// (their slopes' second moments less the squared mean: LEAN mapping, Olano and Baker 2010) and the spectrum's tail finer
// than the finest cascade. A pixel of sea is a rough mirror, not a glass one (Bruneton, Neyret and Holzschuch 2010).
// The slopes are the displaced surface's (each divided by the stretch of its axis), as web-ocean's (`ocean.js` fs).
struct SeaNormal {
  n: vec3<f32>,
  slope: vec2<f32>,
  jac: f32,
  cov: vec3<f32>,
};

// (gx, gy: how far p moves a pixel across and down the screen, metres.)
// (at: the point drawn, frame xz, where the hulls' wakes are read.)
fn ocean_normal(p: vec2<f32>, at: vec2<f32>, gx: vec2<f32>, gy: vec2<f32>) -> SeaNormal {
  var s = vec2<f32>(0.0);
  var cov = vec3<f32>(water.fft.z * 0.5, water.fft.z * 0.5, 0.0);
  var j = vec3<f32>(0.0);
  let sh = shoaling(p);
  // og_core's waves as the surface carries them (shoaled by the seabed), those the pixel cannot hold left to the facets'
  // roughness.
  for (var i = 0u; i < 32u; i++) {
    let a = water.swell[2u * i];
    let b = water.swell[2u * i + 1u];
    let k = length(a.xy);
    let f = shoal_keep(k, sh);
    let keep = pixel_keep(a.xy, gx, gy) * f;
    let th = dot(a.xy, p) + a.w;
    s += a.xy * (a.z * cos(th) * keep);
    let sn = sin(th) * keep;
    j -= vec3<f32>(b.x * a.x, b.y * a.y, b.x * a.y) * sn;
    let ka = k * a.z;
    let d = a.xy / max(k, 1e-6);
    cov += vec3<f32>(d.x * d.x, d.y * d.y, d.x * d.y) * (ka * ka * 0.5 * max(f * f - keep * keep, 0.0));
  }
  if (sh.amp > 0.0) {
    // The shore trains: crests along the distance's lines, travelling inshore (their wave vector k · inshore).
    let k = water.shore[1].z;
    let kv = sh.inshore * k;
    let keep = pixel_keep(kv, gx, gy);
    s += kv * (sh.amp * cos(sh.phase) * keep);
    j -= vec3<f32>(sh.inshore.x * kv.x, sh.inshore.y * kv.y, sh.inshore.x * kv.y) * (sh.lean * sin(sh.phase) * keep);
    let ka = k * sh.amp;
    let d = sh.inshore;
    cov += vec3<f32>(d.x * d.x, d.y * d.y, d.x * d.y) * (ka * ka * 0.5 * max(1.0 - keep * keep, 0.0));
  }
  for (var c = 0u; c < u32(water.fft.x); c++) {
    let uv = cascade_uv(c, p);
    let k = water.cascades[c].x;
    let w = shoal_keep(water.bands[c], sh);
    let first = textureSampleGrad(heights, ripple_sampler, uv, 3u * c + 1u, gx * k, gy * k);
    let second = textureSampleGrad(heights, ripple_sampler, uv, 3u * c + 2u, gx * k, gy * k);
    s += first.xy * w;
    cov += vec3<f32>(max(second.x - first.x * first.x, 0.0), max(second.y - first.y * first.y, 0.0),
      second.z - first.x * first.y) * w * w;
    j += vec3<f32>(first.zw, textureSampleGrad(heights, ripple_sampler, uv, 3u * c, gx * k, gy * k).w) * w;
  }
  var o: SeaNormal;
  // W3: the hulls' wakes' own slopes on top, where the point is drawn.
  o.slope = s / vec2<f32>(max(1.0 + j.x, 0.15), max(1.0 + j.y, 0.15)) + water_wake(at).yz;
  o.n = normalize(vec3<f32>(-o.slope.x, 1.0, -o.slope.y));
  o.jac = (1.0 + j.x) * (1.0 + j.y) - j.z * j.z;
  o.cov = cov;
  return o;
}

// web-ocean's Smith masking for an anisotropic Gaussian of slopes (`lambdaSmith`).
fn smith_lambda(w: vec3<f32>, cov: vec3<f32>) -> f32 {
  let xz2 = max(dot(w.xz, w.xz), 1e-8);
  let il = inverseSqrt(xz2);
  let az = w.xz * il;
  let sig2 = az.x * az.x * cov.x + az.y * az.y * cov.y + 2.0 * az.x * az.y * cov.z;
  let a = max(w.y * il, 1e-3) / sqrt(2.0 * max(sig2, 1e-7));
  if (a >= 1.6) { return 0.0; }
  return (1.0 - 1.259 * a + 0.396 * a * a) / (3.535 * a + 2.181 * a * a);
}

// The sea's foam at rest point p (frame xz) where the surface's Jacobian is `jac`, ported from web-ocean's `ocean.js` fs
// (MIT, third_party/ocean/): the persistent foam of the world levels about the camera (fine near, coarse far), the
// foam of a crest folding now beyond them, aged foam broken into streaks along the wind, all cut by the foam's
// micro-structure into a mask with crisp edges where dense and a soft, translucent margin where thin.
struct SeaFoam {
  cover: f32,
  mask: f32,
  // How fresh the foam is (the levels' g): the bubbles of a crest just broken.
  young: f32,
  // The bubbles' brightness inside it.
  bubbles: f32,
  // W4: how much of the mask is the thin, wet lace an old whitecap leaves (it keeps some of the water's glitter).
  lace: f32,
  // How thick the foam lies here: 0 at a patch's thin edge, 1 in its body.
  thick: f32,
};

// The surf (W1) at rest point p: where the shore trains break, a fresh sheet at each crest's face and its trail
// behind it, fading by the time the next crest comes, and the wash where the water runs thin up the beach.
fn sea_surf(sh: Shoal) -> f32 {
  if (water.shore[0].w < 0.5 || water.shore[2].z <= 0.0) { return 0.0; }
  if (sh.inside <= 0.0) { return 0.0; }
  let half = 3.14159265 / max(water.shore[1].z, 1e-6);
  // Breaking where the trains lean over: inshore of a third of their half length, at the breaking depth.
  let breaking = (1.0 - smoothstep(0.08 * half, 0.4 * half, sh.depth)) * step(1e-4, sh.amp);
  // The time since the crest passed, as a share of a period (the phase falls as time goes on; the crest at pi/2).
  let age = fract((1.5707963 - sh.phase) / 6.2831853);
  // A broken crest runs on inshore as a bore, its face white and the foam it sheds trailing a third of a period behind
  // it; the inner surf zone holds the foam of the bores before it between them (Basco 1985's bore; the surf zone of a
  // beach at any moment is white over most of it, its lines brightest just behind each bore).
  let bore = exp(-age * 2.2) * smoothstep(0.0, 0.03, age);
  let inner = 1.0 - smoothstep(0.05 * half, 0.25 * half, sh.depth);
  let surf = breaking * mix(0.25 * inner, 1.0, bore);
  // The swash: the last bore's water running up and back over the shallowest half metre, all white.
  let wash = (1.0 - smoothstep(0.0, 0.6, sh.depth)) * (0.75 + 0.25 * bore);
  return water.shore[2].z * sh.inside * clamp(max(surf, wash), 0.0, 1.0) * step(0.0, sh.depth - 1e-3);
}

// How far point p lies in the surf zone (1 inshore of the trains' breaking depth, as sea_surf; 0 offshore or with no
// seabed), whatever the surf's foam setting.
fn surf_zone(sh: Shoal) -> f32 {
  if (water.shore[0].w < 0.5) { return 0.0; }
  if (sh.inside <= 0.0) { return 0.0; }
  let half = 3.14159265 / max(water.shore[1].z, 1e-6);
  return sh.inside * (1.0 - smoothstep(0.08 * half, 0.4 * half, sh.depth)) * step(0.0, sh.depth - 1e-3);
}

fn sea_foam(p: vec2<f32>, jac: f32) -> SeaFoam {
  let first = 3u * u32(water.fft.x);
  let l0 = water.foam_levels[0];
  let l1 = water.foam_levels[1];
  let dc = abs(p - frame.view_pos.xz);
  let cheb = max(dc.x, dc.y);
  let w0 = 1.0 - smoothstep(0.30 * l0.w, 0.45 * l0.w, cheb);
  let w1 = 1.0 - smoothstep(0.30 * l1.w, 0.45 * l1.w, cheb);
  let fw0 = textureSample(heights, ripple_sampler, p * l0.x + l0.yz, first);
  let fw1 = textureSample(heights, ripple_sampler, p * l1.x + l1.yz, first + 1u);
  let fwl = mix(fw1, fw0, w0) * w1;
  // (The seabed's shoaling once for all the foam reads of it: two loops over the sea's waves.)
  var sh = OPEN_SEA;
  if (water.shore[0].w > 0.5) { sh = shoaling(p); }
  let surf = sea_surf(sh);
  // The crest's gain is the open sea's (how close its surface lies under the level, water.rs foam_thresholds), kept
  // where the water is deep for the waves that reach the shore (over half their length). Inshore of that the seabed
  // bunches every crest far under the level, and the steep gain whitened the sea from there to the beach: the least
  // gain there (sea.rs FRESH_GAIN), the surf's foam being sea_surf's.
  let shoal_half = 3.14159265 / max(water.shore[1].z, 1e-6);
  let shoals = sh.inside * (1.0 - smoothstep(0.4 * shoal_half, shoal_half, sh.depth));
  let fresh_gain = mix(water.ocean.w, 2.5, shoals);
  // W3: the wakes' foam (their breaking crests and the hulls' sides) is fresh foam too.
  let fresh = max(max(clamp((water.ocean.z - jac) * fresh_gain, 0.0, 1.0), surf), water_wake(p).w);
  let fresh_k = clamp(fwl.g * 2.0 + fresh, 0.0, 1.0);
  // Only the crest breaking now is a solid white (Monahan's stage A, a few tenths of a percent of a gale's sea). The
  // foam it leaves is no longer fed: a sheet of bubbles thinning as they burst, always open between its patches
  // (stage B, most of the whitecap cover), so what the levels hold counts for 0.6 of its cover once it is old.
  // The surf's foam is closed too: the bores churn it over and over, and it lies thick to the beach.
  let zone = surf_zone(sh);
  let solid = clamp(max(max(fresh, smoothstep(0.6, 1.0, fwl.g)), zone), 0.0, 1.0);
  var cover = clamp(max(max(fwl.r * mix(0.8, 1.0, solid), fresh * (1.0 - w1 * 0.7)), surf) + fwl.g * 0.6, 0.0, 1.5);
  let wd = water.sea_wind.xy;
  // W4: the foam's age (the levels' b, mixed as their cover) stretches its structure along the wind as it is carried,
  // from the round lumps of a crest breaking now to streaks some ten times longer than wide (sea_wind.w: foamStreaks).
  let age = select(0.0, fwl.b / max(w1, 1e-3), w1 > 1e-3);
  let stretch = 1.0 + 2.0 * water.sea_wind.w * smoothstep(1.0, 8.0, age);
  let ws = vec2<f32>(dot(p, wd) / stretch, dot(p, vec2<f32>(-wd.y, wd.x))) * vec2<f32>(0.035, 0.11) + water.foam_uv[1].zw;
  let streak = textureSample(heights, ripple_sampler, ws, first + 2u).a;
  // The old foam's structure. On the open sea, the age's streaks along the wind, faded to their mean where a pixel
  // spans their period across it (some 0.6 m; past that they alias into a regular comb along the wind). In the surf
  // zone, broad churning masses with the backwash's dark gaps between them: the surf's foam is churned by the bores,
  // not drawn out by the wind (its age's streaks made the whole band a comb).
  let streak_mod = mix(smoothstep(0.25, 0.75, streak) * 1.5, 0.75, smoothstep(0.3, 0.8, fwidthFine(ws.y) * 16.0));
  let masses = smoothstep(0.3, 0.7, foam_noise(p * 0.15) * 0.65 + foam_noise(p * 0.47 + vec2<f32>(3.1, 7.9)) * 0.35) * 1.4;
  cover *= mix(mix(streak_mod, masses, zone), 1.0, fresh_k * (1.0 - 0.6 * zone));
  let pat = textureSample(heights, ripple_sampler, p * 0.23 + water.foam_uv[0].xy, first + 2u).r * 0.78
    + textureSample(heights, ripple_sampler, p * 0.71 + water.foam_uv[0].zw, first + 2u).g * 0.22;
  let fw = fwidthFine(pat) + 0.03;
  let soft = 0.04 + 0.14 * (1.0 - clamp(cover, 0.0, 1.0));
  // The pattern's cut. Old foam never closes: even where the levels hold all they can (the gain is steep, so the
  // cover's own edge is a step and a full cover drew each patch as a flat cut-out) a seventh of the pattern stays
  // open, the patch's rim and the holes in its body the pattern's. The crest breaking now is closed all over.
  let cut = 1.0 - cover * mix(0.85, 1.0, solid);
  var fm = smoothstep(cut - fw - soft, cut + fw + soft, pat);
  // Where a pixel spans the pattern (its change across the pixel near its range), the mask is what the pattern's cut
  // covers of the pixel on average, the share of the lace over 1 - cover: about the cover itself. The cut softened by
  // fwidth alone drew distant foam as a grey sheet wherever there was any.
  fm = mix(fm, clamp(cover, 0.0, 1.0), smoothstep(0.15, 0.4, fw));
  var o: SeaFoam;
  o.cover = cover;
  // How far the pattern stands over its cut: a patch's rim is a single layer of bubbles, its body a pile of them
  // (the cover itself where a pixel spans the pattern).
  o.thick = max(mix(smoothstep(0.0, 0.4, pat - cut), clamp(cover, 0.0, 1.0), smoothstep(0.15, 0.4, fw)), zone);
  o.mask = clamp(fm * smoothstep(0.0, 0.5, cover), 0.0, 1.0) * (0.68 + 0.32 * smoothstep(0.3, 0.9, cover));
  // A thin rim lets the water show through it.
  o.mask *= mix(0.75 + 0.25 * o.thick, 1.0, solid);
  o.bubbles = textureSample(heights, ripple_sampler, p * 1.9 + water.foam_uv[1].xy, first + 2u).g;
  o.mask = max(o.mask, smoothstep(0.35, 0.9, fwl.g) * smoothstep(0.25, 0.6, pat + 0.25) * 0.95);
  // The lace (the levels' a) where the dense cover has gone: a thin web of the finer lace along the streaks, its share
  // of a pixel the lace's own where the pixel spans the web.
  // Only past force 7 (13.9 m/s), where Beaufort's sea first has foam blown in streaks, and well marked by force 8.
  let lace_amount = clamp(fwl.a - fwl.r, 0.0, 1.0) * water.sea_wind.w * smoothstep(12.0, 18.0, water.sea_wind.z);
  // The lace's pattern: threads along the wind some 3.5 m apart, meandering with the streak noise sampled slowly both
  // ways (so each thread holds its line), their edges broken by the finer lace's bubbles, cut where they cover the
  // share of the surface the lace holds. The threads' triangle wave is uniform on [0, 1], so the cut at 1 - share
  // covers about that share (a mix of the noise channels, bunched about its mean, was all but never cut).
  let wl = textureSample(heights, ripple_sampler, ws * vec2<f32>(0.12, 0.014), first + 2u).a;
  let wm = textureSample(heights, ripple_sampler, ws * vec2<f32>(0.3, 0.11) + 0.37, first + 2u).a;
  let across_m = dot(p, vec2<f32>(-wd.y, wd.x));
  let along_m = dot(p, wd);
  // (An irregular spacing too: a warp across the wind, its slope under the threads' own so each stays one line.)
  let wi = textureSample(heights, ripple_sampler, vec2<f32>(along_m * 0.002, across_m * 0.0125) + 0.71, first + 2u).a;
  // And bent: a wander across the wind some 12 m long along it, so a streak winds as drifting foam does instead of
  // running ruler-straight (its slope across the wind still under the threads').
  let wb = textureSample(heights, ripple_sampler, vec2<f32>(along_m * 0.02, across_m * 0.003) + 0.13, first + 2u).a;
  let x = across_m / 3.5 + wl * 2.2 + wm * 0.45 + (wi - 0.5) * 0.5 + (wb - 0.5) * 3.0;
  let web = textureSample(heights, ripple_sampler, p * 0.53 + water.foam_uv[0].zw, first + 2u).g;
  let threads = abs(fract(x) - 0.5) * 2.0 + (web - 0.75) * 0.08;
  // Each thread its own: some strong, some faint, and broken along the wind into lengths of some tens of metres (the
  // threads' centres at whole x).
  let r = spray_rand(spray_hash(bitcast<u32>(i32(floor(x + 0.5))) + 0x2545f491u));
  let seg = textureSample(heights, ripple_sampler, vec2<f32>(along_m * 0.004 + r * 7.0, r * 3.1), first + 2u).a;
  let lw = fwidthFine(x) * 2.0 + 0.05;
  // (Old foam, the whitecaps' residue, lies over two to three times the whitecaps' share: some 20 to 30 % of a gale's
  // sea against their 10 %, Monahan and O'Muircheartaigh's stage B; the threads' weights and lengths keep 0.3 of it.)
  let lace_cover = clamp(lace_amount * 3.0, 0.0, 1.0);
  // (A thread 0.4 of the spacing at most: the streaks are narrow, a metre or so.)
  let share = 0.4 * lace_cover * clamp(r * 1.6, 0.0, 1.0) * smoothstep(0.48, 0.62, seg);
  // Soft-edged: a streak is drifting foam thinning to its sides, not a line.
  let lsoft = lw + 0.12;
  var lace = smoothstep(1.0 - share - lsoft, 1.0 - share + lsoft, threads);
  // Where a pixel spans the threads, the mean of their shares.
  lace = mix(lace, lace_cover * 0.12, smoothstep(0.15, 0.4, lw));
  // Thinner and thicker along its length, over some metres (holes cut in a thin thread read as beads, and the
  // foam sheets' finer pattern drew pores along it).
  let thick = textureSample(heights, ripple_sampler, vec2<f32>(along_m * 0.03, across_m * 0.08) + 0.29, first + 2u).a;
  lace *= mix(0.3, 1.0, smoothstep(0.3, 0.7, thick));
  // (None where the lace holds none: the bubbles lift the threads' top past 1, which the cut at 1 would draw.)
  lace *= smoothstep(0.0, 0.05, lace_cover);
  // Faint: the old foam's film over the dark sea, well under a whitecap's cover.
  o.lace = lace * 0.2 * (1.0 - o.mask);
  o.mask = max(o.mask, lace * 0.2);
  o.young = fwl.g;
  return o;
}

// Bruneton et al. 2010's mean Fresnel over a rough sea's facets: the grazing mirror dims as the slopes it averages
// spread (sigma: their deviation along the view).
fn sea_fresnel(cos_v: f32, sigma: f32) -> f32 {
  return pow(1.0 - cos_v, 5.0 * exp(-2.69 * sigma)) / (1.0 + 22.7 * pow(sigma, 1.5));
}

// How many metres of the surface a pixel spans at `world` along the view's slant: the distance over the projection's
// scale, stretched by the grazing angle. Smooth across the grid's triangles, where screen derivatives of the rest
// point jump at every edge (and drew the edge as a line where they faded the waves).
fn sea_footprint(world: vec3<f32>) -> f32 {
  let to_eye = frame.view_pos.xyz - world;
  let dist = length(to_eye);
  let scale = length(vec3<f32>(frame.view_proj[0].y, frame.view_proj[1].y, frame.view_proj[2].y));
  let pixel = 2.0 / max(scale * water.screen.y, 1e-6);
  return dist * pixel / max(abs(to_eye.y) / max(dist, 1e-6), 0.02);
}

// A vertex of the sea's clipmap: its rest point (frame xz) and the spacing its waves are read at, or w < 0 for a
// vertex of a cell left out (the hole a level leaves for the one inside it).
//
// Every level is `s` × `s` cells about its centre; each level past the first leaves out the cells under the level
// inside it, whose square lies on its lattice (water.rs `clipmap`). Towards its outer edge a level morphs onto the
// next one's lattice: its odd vertices take the mean of their even neighbours' displacement (along the edge, or the
// cells' diagonal for a vertex odd both ways), and every vertex reads the waves at a spacing growing from its own
// cell to the next level's, so the edge it shares with the next level's hole is that level's, vertex for vertex.
// Past the outermost level a ring of quads reaches the horizon (clip.z), flat at its far edge.
// A bounded sea's rest point (not endless): clamped to its rectangle in the surface's frame, so the clipmap's cells
// past it fold onto its edge.
fn sea_clamp(p: vec2<f32>) -> vec2<f32> {
  if (water.clip.w > 0.5) { return p; }
  let local = (water.inverse * vec4<f32>(p.x, water.model[3].y, p.y, 1.0)).xz;
  let h = water.size.xy * 0.5;
  return (water.model * vec4<f32>(clamp(local, -h, h).x, 0.0, clamp(local, -h, h).y, 1.0)).xz;
}

struct SeaVertex {
  rest: vec2<f32>,
  d: vec3<f32>,
  skip: bool,
};

fn sea_vertex(vi: u32) -> SeaVertex {
  var corners = array<vec2<f32>, 6>(
    vec2<f32>(0.0, 0.0), vec2<f32>(0.0, 1.0), vec2<f32>(1.0, 1.0),
    vec2<f32>(0.0, 0.0), vec2<f32>(1.0, 1.0), vec2<f32>(1.0, 0.0));
  let s = u32(water.clip.y);
  let per_level = s * s * 6u;
  let levels = u32(water.clip.x);
  let level = vi / per_level;
  var o: SeaVertex;
  o.skip = false;
  if (level >= levels) {
    // The horizon ring: the outermost level's edge (its even vertices) to the ring's radius.
    let k = vi - levels * per_level;
    let corner = corners[k % 6u];
    let half = s / 2u;
    let t = (k / 6u + u32(corner.x)) % (4u * half);
    let last = water.levels[levels - 1u];
    let h = f32(half) * last.z;
    let along = f32(t % half) * 2.0 * last.z;
    var q: vec2<f32>;
    switch (t / half) {
      case 0u: { q = vec2<f32>(-h + along, -h); }
      case 1u: { q = vec2<f32>(h, -h + along); }
      case 2u: { q = vec2<f32>(h - along, h); }
      default: { q = vec2<f32>(-h, h - along); }
    }
    if (corner.y > 0.5) {
      o.rest = sea_clamp(last.xy + q * max(water.clip.z / h, 1.0));
      o.d = vec3<f32>(0.0);
    } else {
      o.rest = sea_clamp(last.xy + q);
      o.d = ocean_displace(o.rest, 2.0 * last.z);
    }
    return o;
  }
  let k = vi % per_level;
  let quad = k / 6u;
  let cell_ij = vec2<f32>(f32(quad % s), f32(quad / s));
  let lv = water.levels[level];
  let c = lv.z;
  let half = f32(s) * 0.5;
  if (level > 0u) {
    let inner = water.levels[level - 1u];
    let middle = lv.xy + (cell_ij + 0.5 - half) * c;
    if (all(abs(middle - inner.xy) < vec2<f32>(half * 0.5 * c))) {
      o.skip = true;
      return o;
    }
  }
  let g = cell_ij + corners[k % 6u] - half;
  let p = lv.xy + g * c;
  o.rest = sea_clamp(p);
  let m = smoothstep(0.7, 0.95, max(abs(g.x), abs(g.y)) / half);
  let spacing = c * (1.0 + m);
  let odd = vec2<bool>(abs(g.x) % 2.0 > 0.5, abs(g.y) % 2.0 > 0.5);
  var d = ocean_displace(o.rest, spacing);
  if (m > 0.0 && (odd.x || odd.y)) {
    let e = vec2<f32>(select(0.0, c, odd.x), select(0.0, c, odd.y));
    let mean = 0.5 * (ocean_displace(sea_clamp(p - e), spacing) + ocean_displace(sea_clamp(p + e), spacing));
    d = mix(d, mean, m);
  }
  o.d = d;
  return o;
}

// The drawn sea's height probe (water.rs `WaterRenderer::probe_sea`, Renderer::sea_heights): per point (frame xz),
// the height the clipmap's vertices draw there (the rest point found by undoing the whole surface's lean, at the
// spacing sea_vertex reads its level's vertices at: its cell, growing to twice at its edge); the height of the sea's
// own function there as the GPU evaluates it (every wave of og_core::ocean, its rest point found as the tick finds it:
// Ocean::height on the GPU); what the drawn surface differs from that by (the cascades, the waves the vertices'
// spacing leaves out, and the cascades' lean moving the rest point); and the hulls' wakes' share.
@group(2) @binding(0) var<storage, read> probe_in: array<vec2<f32>>;
@group(2) @binding(1) var<storage, read_write> probe_out: array<vec4<f32>>;

fn probe_spacing(p: vec2<f32>) -> f32 {
  let half = water.clip.y * 0.5;
  let levels = u32(water.clip.x);
  for (var l = 0u; l < levels; l++) {
    let lv = water.levels[l];
    let d = max(abs(p.x - lv.x), abs(p.y - lv.y)) / (half * lv.z);
    if (d <= 1.0) { return lv.z * (1.0 + smoothstep(0.7, 0.95, d)); }
  }
  return 2.0 * water.levels[levels - 1u].z;
}

@compute @workgroup_size(64)
fn probe_sea(@builtin(global_invocation_id) id: vec3<u32>) {
  if (id.x >= arrayLength(&probe_in)) { return; }
  let goal = probe_in[id.x];
  var rest = goal;
  for (var i = 0; i < 8; i++) {
    rest = goal - ocean_displace(rest, probe_spacing(rest)).xz;
  }
  let cell = probe_spacing(rest);
  let sh = shoaling(rest);
  let drawn = ocean_swell(rest, cell, sh).y + ocean_cascades(rest, cell, sh).y;
  var own = goal;
  for (var i = 0; i < 8; i++) {
    own = goal - ocean_swell(own, 0.0, shoaling(own)).xz;
  }
  let swell = ocean_swell(own, 0.0, shoaling(own)).y;
  let wake = water_wake(goal).x;
  probe_out[id.x] = vec4<f32>(water.model[3].y + drawn + wake, swell, drawn - swell, wake);
}

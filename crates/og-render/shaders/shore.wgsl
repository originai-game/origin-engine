// The seabed under a sea (W1, og_core::shore and og_core::ocean::Ocean::shoaling), shared by water.wgsl and
// ocean_foam.wgsl: prepended to each, which declares `shore_tex` (the field's texels: depth, signed distance to the
// waterline, its gradient) and the accessors `sea_shore(i)` (the three rows of water.rs `shore_gpu`) and `sea_wave(i)`
// (the tick's waves, two rows each, as WaterUniform.swell).

// The seabed at frame point p (og_core::shore::Shore::sample, the same texels filtered the same way): inside < 0
// outside its grid (or without one).
struct ShoreAt {
  depth: f32,
  distance: f32,
  inshore: vec2<f32>,
  coherence: f32,
  inside: f32,
};

fn shore_at(p: vec2<f32>) -> ShoreAt {
  var o = ShoreAt(0.0, 0.0, vec2<f32>(0.0), 0.0, -1.0);
  if (sea_shore(0u).w < 0.5) { return o; }
  let g = (p - sea_shore(0u).xy) / sea_shore(0u).z;
  let hx = sea_shore(1u).x - 1.0;
  let hz = sea_shore(1u).y - 1.0;
  if (!(g.x >= 0.0 && g.y >= 0.0 && g.x <= hx && g.y <= hz)) { return o; }
  let i = vec2<f32>(min(floor(g.x), hx - 1.0), min(floor(g.y), hz - 1.0));
  let t = g - i;
  let ii = vec2<i32>(i);
  let a = textureLoad(shore_tex, ii, 0);
  let b = textureLoad(shore_tex, ii + vec2<i32>(1, 0), 0);
  let c = textureLoad(shore_tex, ii + vec2<i32>(0, 1), 0);
  let d = textureLoad(shore_tex, ii + vec2<i32>(1, 1), 0);
  let lo = a + (b - a) * t.x;
  let hi = c + (d - c) * t.x;
  let v = lo + (hi - lo) * t.y;
  let len = length(v.zw);
  o.depth = v.x;
  o.distance = v.y;
  o.inshore = select(vec2<f32>(0.0), -v.zw / len, len > 1e-6);
  o.coherence = smoothstep(0.5, 0.9, len);
  o.inside = smoothstep(0.0, 8.0, min(min(g.x, g.y), min(hx - g.x, hz - g.y)));
  return o;
}

// What the seabed under rest point p does to the sea there (og_core::ocean::Ocean::shoaling): each wave of wavenumber
// k keeps shoal_keep(k, ...) of itself (tanh(k d), capped with the rest at the breaking height, a significant height of
// 0.78 d), and the shore trains roll in with `amp` and `lean` along `inshore` at `phase`.
struct Shoal {
  depth: f32,
  inside: f32,
  cap: f32,
  amp: f32,
  lean: f32,
  inshore: vec2<f32>,
  phase: f32,
};

const OPEN_SEA: Shoal = Shoal(0.0, 0.0, 1.0, 0.0, 0.0, vec2<f32>(0.0), 0.0);

fn shoal_keep(k: f32, sh: Shoal) -> f32 {
  return (1.0 + (tanh(k * sh.depth) - 1.0) * sh.inside) * sh.cap;
}

fn shoaling(p: vec2<f32>) -> Shoal {
  let sp = shore_at(p);
  if (sp.inside < 0.0) { return OPEN_SEA; }
  var o = OPEN_SEA;
  o.depth = max(sp.depth, 0.0);
  o.inside = sp.inside;
  var v = 0.0;
  for (var i = 0u; i < 32u; i++) {
    let a = sea_wave(2u * i);
    let f = shoal_keep(length(a.xy), o);
    v += (a.z * f) * (a.z * f) * 0.5;
  }
  let k = sea_shore(1u).z;
  let half = 3.14159265 / max(k, 1e-6);
  let a_s = sea_shore(1u).w * (1.0 - smoothstep(0.5 * half, half, o.depth)) * sp.coherence * sp.inside;
  v += a_s * a_s * 0.5;
  let hs = 4.0 * sqrt(v);
  let limit = 0.78 * o.depth;
  if (hs > limit) { o.cap = 1.0 + (limit / hs - 1.0) * sp.inside; }
  var steep = 0.0;
  for (var i = 0u; i < 32u; i++) {
    let a = sea_wave(2u * i);
    steep += sea_wave(2u * i + 1u).z * shoal_keep(length(a.xy), o);
  }
  o.amp = a_s * o.cap;
  o.phase = sea_shore(2u).y - k * sp.distance;
  if (o.amp > 0.0) {
    let q = min(0.9 * (1.0 - smoothstep(0.1 * half, half, o.depth)), max(0.95 - steep, 0.0));
    o.lean = min(q / k, 3.0 * o.amp);
    o.inshore = sp.inshore;
  }
  return o;
}


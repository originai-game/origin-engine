// The physical sky (sky_physical.rs, the R4 round): single scattering by
// air (Rayleigh) and aerosols (Mie) with ozone absorption in a spherical
// atmosphere, plus Hillaire's multiple-scattering term from the table the
// CPU integrated. The same formulas as `PhysicalAtmosphere::radiance`.
// SCATTER stands for the block (`PhysicalAtmosphere::gpu`): the renderer
// replaces it with `frame.sky_scatter` in the sky pass and with
// `params.scatter` in the image-based light's sky.
//   [0] rgb: Rayleigh scattering per km, w: 1 when on
//   [1] x: Mie scattering per km, y: Mie extinction, z: 1 / Rayleigh height, w: 1 / Mie height
//   [2] rgb: ozone absorption at its peak, w: Mie g
//   [3] x: planet radius (km), y: top of the air, z: the viewer's altitude (km), w: the radiance scale
//   [4] rgb: ground albedo
//   [5] rgb: the night sky's radiance
//   [6..38] the multiple-scattering table: 16 sun zenith cosines at 0 km, then at 8 km

const SCATTER_PI: f32 = 3.14159265;
const SCATTER_MS: u32 = 16u;

fn scatter_on() -> bool { return SCATTER[0].w > 0.5; }

// Where a ray from `o` along unit `d` meets the sphere of radius `r` about
// the origin: (near, far, 1), or z 0 when it misses.
fn scatter_sphere(o: vec3<f32>, d: vec3<f32>, r: f32) -> vec3<f32> {
  let b = dot(o, d);
  let c = dot(o, o) - r * r;
  let disc = b * b - c;
  if (disc < 0.0) { return vec3<f32>(0.0); }
  let s = sqrt(disc);
  return vec3<f32>(-b - s, -b + s, 1.0);
}

// The Chapman column above the horizon (sky_physical.rs `chapman`).
fn scatter_chapman_up(x: f32, mu: f32) -> f32 {
  let y = sqrt(0.5 * x) * mu;
  return sqrt(0.5 * SCATTER_PI * x) * 2.911 / (1.911 * sqrt(SCATTER_PI) * y + sqrt(SCATTER_PI * y * y + 8.474));
}

fn scatter_chapman(r: f32, mu: f32, h: f32, bottom: f32) -> f32 {
  let x = r / h;
  let a = exp(min((bottom - r) / h, 0.0));
  if (mu >= 0.0) { return scatter_chapman_up(x, mu) * a; }
  let r0 = r * sqrt(max(1.0 - mu * mu, 0.0));
  return 2.0 * scatter_chapman_up(r0 / h, 0.0) * exp(min((bottom - r0) / h, 0.0)) - scatter_chapman_up(x, -mu) * a;
}

// The ozone column (km at peak density) from radius `r` along zenith cosine `mu`.
fn scatter_ozone_column(r: f32, mu: f32, bottom: f32) -> f32 {
  let h = r - bottom;
  var above = 0.0;
  if (h <= 10.0) {
    above = 15.0;
  } else if (h < 25.0) {
    let u = (h - 10.0) / 15.0;
    above = 15.0 * (1.0 - 0.5 * u * u);
  } else if (h < 40.0) {
    let u = (40.0 - h) / 15.0;
    above = 7.5 * u * u;
  }
  let rs = max(r, bottom + 25.0);
  let s = r * r * (1.0 - mu * mu);
  let slant = rs / sqrt(max(rs * rs - s, rs * rs * 6e-4));
  return above * min(slant, 40.0);
}

// The sunlight reaching radius `r` with the sun at zenith cosine `mu`.
fn scatter_sun_transmittance(r: f32, mu_in: f32) -> vec3<f32> {
  let bottom = SCATTER[3].x;
  let k = min(bottom / r, 1.0);
  let horizon = -sqrt(max(1.0 - k * k, 0.0));
  let lit = smoothstep(horizon - 0.006, horizon + 0.003, mu_in);
  if (lit <= 0.0) { return vec3<f32>(0.0); }
  let mu = max(mu_in, horizon + 0.003);
  let hr = 1.0 / SCATTER[1].z;
  let hm = 1.0 / SCATTER[1].w;
  let depth = SCATTER[0].rgb * (hr * scatter_chapman(r, mu, hr, bottom))
    + vec3<f32>(SCATTER[1].y * hm * scatter_chapman(r, mu, hm, bottom))
    + SCATTER[2].rgb * scatter_ozone_column(r, mu, bottom);
  return exp(-depth) * lit;
}

// Ψ_ms at height `h` km and sun zenith cosine `mu`.
fn scatter_ms(h: f32, mu: f32) -> vec3<f32> {
  let x = (clamp(mu, -1.0, 1.0) * 0.5 + 0.5) * f32(SCATTER_MS - 1u);
  let i = min(u32(floor(x)), SCATTER_MS - 2u);
  let f = x - f32(i);
  let lo = mix(SCATTER[6u + i], SCATTER[7u + i], f).rgb;
  let hi = mix(SCATTER[6u + SCATTER_MS + i], SCATTER[7u + SCATTER_MS + i], f).rgb;
  return mix(lo, hi, clamp(h / 8.0, 0.0, 1.0));
}

// The Rayleigh and Mie (Cornette–Shanks) phase functions for the cosine
// `nu` between the view and the sun.
fn scatter_phases(nu: f32) -> vec2<f32> {
  let g = SCATTER[2].w;
  let g2 = g * g;
  let pr = 3.0 / (16.0 * SCATTER_PI) * (1.0 + nu * nu);
  let pm = 3.0 / (8.0 * SCATTER_PI) * (1.0 - g2) * (1.0 + nu * nu)
    / ((2.0 + g2) * pow(max(1.0 + g2 - 2.0 * g * nu, 1e-6), 1.5));
  return vec2<f32>(pr, pm);
}

// One sample of the air at `p` (km, planet-centred), the sun along unit
// `sun`, `phases` scatter_phases: the light it scatters towards the eye
// per km, of the sun (single scattering) and of the sky (Hillaire's
// multiple scattering), and its extinction per km.
struct ScatterSample { sun: vec3<f32>, sky: vec3<f32>, ext: vec3<f32> };

fn scatter_at(p: vec3<f32>, sun: vec3<f32>, phases: vec2<f32>) -> ScatterSample {
  let bottom = SCATTER[3].x;
  let r = length(p);
  // Never below the ground: the flat world's surfaces sit above the curved
  // planet's, and the sea floor's a little below it.
  let h = max(r - bottom, 0.0);
  let mu = dot(p, sun) / r;
  let sr = SCATTER[0].rgb * exp(-h * SCATTER[1].z);
  let dm = exp(-h * SCATTER[1].w);
  let sm = SCATTER[1].x * dm;
  let oz = max(1.0 - abs(h - 25.0) / 15.0, 0.0);
  var o: ScatterSample;
  o.ext = max(sr + vec3<f32>(SCATTER[1].y * dm) + SCATTER[2].rgb * oz, vec3<f32>(1e-7));
  o.sun = (sr * phases.x + vec3<f32>(sm * phases.y)) * scatter_sun_transmittance(max(r, bottom), mu);
  o.sky = (sr + vec3<f32>(sm)) * scatter_ms(h, mu);
  return o;
}

// The air between the eye `h0` km up and a point `t` km along unit `d`
// (Hillaire 2020 §5.5, aerial perspective): the light it scatters towards
// the eye (the sun along unit `sun` with illuminance 1, in the engine's
// units, as physical_sky), of the sun and of the sky apart (clouds shade
// them differently), and what it lets through from behind.
struct AirPath { sun: vec3<f32>, sky: vec3<f32>, through: vec3<f32> };

fn air_path(d: vec3<f32>, sun: vec3<f32>, h0: f32, t: f32, steps: u32) -> AirPath {
  let o = vec3<f32>(0.0, SCATTER[3].x + max(h0, 0.001), 0.0);
  let phases = scatter_phases(dot(d, sun));
  let dt = max(t, 0.0) / f32(steps);
  var out: AirPath;
  out.sun = vec3<f32>(0.0);
  out.sky = vec3<f32>(0.0);
  out.through = vec3<f32>(1.0);
  for (var i = 0u; i < steps; i++) {
    let x = scatter_at(o + d * (dt * (f32(i) + 0.5)), sun, phases);
    let seg = exp(-x.ext * dt);
    let k = out.through * (1.0 - seg) / x.ext;
    out.sun += k * x.sun;
    out.sky += k * x.sky;
    out.through *= seg;
  }
  out.sun *= SCATTER[3].w;
  out.sky *= SCATTER[3].w;
  return out;
}

// The sky's radiance along unit `d` from `h0` km up, the sun along unit
// `sun` with illuminance 1, in the engine's units, at `steps` samples:
// single and multiple scattering and the lit ground (no sun disc).
fn physical_sky(d: vec3<f32>, sun: vec3<f32>, h0: f32, steps: u32) -> vec3<f32> {
  let bottom = SCATTER[3].x;
  let o = vec3<f32>(0.0, bottom + max(h0, 0.001), 0.0);
  let top = scatter_sphere(o, d, SCATTER[3].y);
  if (top.z < 0.5) { return vec3<f32>(0.0); }
  let ground = scatter_sphere(o, d, bottom);
  let hits = ground.z > 0.5 && ground.x > 0.0;
  let start = max(top.x, 0.0);
  let end = select(top.y, ground.x, hits);
  if (end <= start) { return vec3<f32>(0.0); }
  let phases = scatter_phases(dot(d, sun));
  let len = end - start;
  let n = f32(steps);
  var l = vec3<f32>(0.0);
  var t = vec3<f32>(1.0);
  for (var i = 0u; i < steps; i++) {
    let a = f32(i) / n;
    let b = f32(i + 1u) / n;
    let dt = (b * b - a * a) * len;
    let x = scatter_at(o + d * (start + (a * a + b * b) * 0.5 * len), sun, phases);
    let seg = exp(-x.ext * dt);
    l += t * (x.sun + x.sky) * (1.0 - seg) / x.ext;
    t *= seg;
  }
  if (hits) {
    let p = o + d * ground.x;
    let mu = dot(p / length(p), sun);
    l += t * scatter_sun_transmittance(bottom, mu) * max(mu, 0.0) * SCATTER[4].rgb / SCATTER_PI;
  }
  return l * SCATTER[3].w;
}

// The sun's light seen along `d` from `h0` km up (the disc's colour).
fn physical_sun(d: vec3<f32>, h0: f32) -> vec3<f32> {
  return scatter_sun_transmittance(SCATTER[3].x + max(h0, 0.001), d.y);
}

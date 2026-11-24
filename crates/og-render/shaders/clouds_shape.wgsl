// The clouds' shape (clouds.rs): the baked fields and the density they
// give a point of the layer. Before sky.wgsl in the sky's module and
// before cloud_shadow_bake.wgsl in the shadow bake's.
//
// The model is Schneider's dimensional profile and up-rez ("Nubis,
// Evolved", SIGGRAPH 2022; "Nubis³", SIGGRAPH 2023;
// third_party/nubis-clouds.md), on fields of Origin's own:
//   - the map says, for a column of the layer, how far inside a cloud it
//     stands (0 at the cloud's edge, 1 in its core) and how tall the cloud
//     is there (cloud_column);
//   - the kind's envelope rounds the cloud: widest a little over its
//     base and narrowing to its top, as a dome does, so its sides lean
//     and no column of the map is drawn straight up as a wall; with the
//     column's it makes the dimensional profile: 0 at the cloud's skin,
//     rising inward (cloud_profile);
//   - the noise erodes the profile: the cloud is where the profile passes
//     the noise, so the billows' round middles stand out of the surface
//     and the creases between them cut into it, with a soft Nubis density
//     ramp and height-based base wispiness (cloud_density_from).
// frame.clouds: [0] coverage, density, base, thickness; [1] the wind's
// offset, the scale; [2].w detail; [10].xy the viewer's place; [11].w wind
// heading (radians); [12] the kind's envelope: girth, height, top power,
// anvil reach; [13] kind field selector, profile edge width, erosion,
// height variation; [14] base raggedness, under-girth taper, high layer xz.

// The shape volume and its repeating sampler, the detail volume and the
// map (clouds.rs, clouds_noise.wgsl).
@group(1) @binding(1) var cloud_noise: texture_3d<f32>;
@group(1) @binding(2) var cloud_sampler: sampler;
@group(1) @binding(7) var cloud_detail: texture_3d<f32>;
@group(1) @binding(8) var cloud_map: texture_2d<f32>;

// Characteristic softness of a cloud's boundary in the eroded profile (0 to 1),
// shared by the shadow bake (cloud_edge_in) and raymarcher base edge. Softer
// than a clay shell so billow edges and wispy tops feather naturally into air.
const CLOUD_EDGE: f32 = 0.16;

fn cloud_remap(v: f32, lo: f32, hi: f32) -> f32 {
  return clamp((v - lo) / max(hi - lo, 1e-4), 0.0, 1.0);
}

// `p`'s point in the field (metres: the world's x, the height, the world's
// z), `p` planet-centred with the viewer over the origin's xz, `h` its
// height above the ground: the clouds stand over the world (the viewer's
// place, [10].xy) and go with the wind (its offset, [1]), each wrapped at
// the field's period, four tiles of the scale, which every scale below
// repeats in.
fn cloud_noise_at(p: vec3<f32>, h: f32) -> vec3<f32> {
  let c = frame.clouds;
  return vec3<f32>(p.x + c[10].x, h, p.z + c[10].y) - c[1].xyz;
}

// `t` at `uv` through a cubic B-spline of its texels (Sigg and Hadwiger,
// "Fast Third-Order Texture Filtering", GPU Gems 2, 2005: four reads
// between texels): its slope has no step from one texel to the next.
fn cloud_cubic(t: texture_2d<f32>, uv: vec2<f32>) -> vec4<f32> {
  let size = vec2<f32>(textureDimensions(t));
  let st = uv * size - 0.5;
  let i = floor(st);
  let f = st - i;
  let f2 = f * f;
  let f3 = f2 * f;
  let w0 = (1.0 - 3.0 * f + 3.0 * f2 - f3) / 6.0;
  let w1 = (4.0 - 6.0 * f2 + 3.0 * f3) / 6.0;
  let w3 = f3 / 6.0;
  let g0 = w0 + w1;
  let g1 = 1.0 - g0;
  let h0 = (i - 0.5 + w1 / g0) / size;
  let h1 = (i + 1.5 + w3 / g1) / size;
  return g0.y * (g0.x * textureSampleLevel(t, cloud_sampler, vec2<f32>(h0.x, h0.y), 0.0) + g1.x * textureSampleLevel(t, cloud_sampler, vec2<f32>(h1.x, h0.y), 0.0))
    + g1.y * (g0.x * textureSampleLevel(t, cloud_sampler, vec2<f32>(h0.x, h1.y), 0.0) + g1.x * textureSampleLevel(t, cloud_sampler, vec2<f32>(h1.x, h1.y), 0.0));
}

// The map over the field's point `at`: every read of the map, the march's,
// the light's and the shadow bake's, so they are of one cloud.
fn cloud_map_at(at: vec3<f32>, exact: bool) -> vec4<f32> {
  let uv = at.xz / (4.0 * frame.clouds[1].w);
  if (!exact) { return textureSampleLevel(cloud_map, cloud_sampler, uv, 0.0); }
  return cloud_cubic(cloud_map, uv);
}

// How much of the map's field stands in cloud for a cover of `c0`, the
// profile `edge` wide: the field is uniform-ish, the noise erodes the outer
// fifth of the profile on average, so a little more than `c0`; an overcast
// stands everywhere, in every column's core.
fn cloud_stand(c0: f32, edge: f32) -> f32 {
  return mix(c0 + 0.2 * edge * min(c0 * 10.0, 1.0), 1.0 + edge, smoothstep(0.85, 1.0, c0));
}

// A column of the layer at one height.
struct CloudColumn {
  // How far inside a cloud it stands: 0 at the cloud's edge and outside
  // it, 1 in its core.
  inside: f32,
  // How far up the layer the cloud stands in its core (0 to 1).
  height: f32,
  // How far up the layer the cloud's envelope stands in this column.
  top: f32,
  // The share of the layer over the base in which the cloud becomes whole
  // here: how low it hangs.
  base: f32,
};

// The column of the map's texel `m`, `hf` of the way up the layer: a
// cloud is as tall as the map says, its envelope round (cloud_profile),
// and a vigorous tower of a kind with an anvil spreads out under the
// layer's top — gated by tower presence so anvils never float detached
// over clear sky.
fn cloud_column(m: vec4<f32>, hf: f32) -> CloudColumn {
  let c = frame.clouds;
  let tops = c[12];
  let kind = c[13];
  let field = mix(m.r, m.b, kind.x);
  let stand = cloud_stand(c[0].x, kind.y);
  let raw_inside = (field - 1.0 + stand) / kind.y;
  let base_inside = clamp(raw_inside, 0.0, 1.0);
  // Anvil spreads only around genuine convective towers (never in clear air).
  let tower_mask = smoothstep(-0.35, 0.25, raw_inside);
  let reach = tops.w * m.a * tower_mask * smoothstep(0.62, 0.88, hf);
  let inside = clamp(base_inside + reach, 0.0, 1.0);
  let height = tops.y * mix(1.0, m.g, kind.w);
  let girth = tops.x * height;
  let out = 1.0 - inside;
  let top = girth + (height - girth) * pow(max(1.0 - out * out, 0.0), 1.0 / tops.z);
  let base = c[14].x * height * (0.4 + 1.2 * mix(m.b, m.r, kind.x));
  return CloudColumn(inside, height, top, base);
}

// How far in from a cloud's edge its columns hold the cloud's own density
// (of `inside`).
const CLOUD_MARGIN: f32 = 0.3;

fn cloud_thin(col: CloudColumn) -> f32 {
  return smoothstep(0.0, CLOUD_MARGIN, col.inside);
}

// How far in from a cloud's side the profile is whole (of `inside`).
const CLOUD_SIDE: f32 = 0.7;

fn cloud_ease(x: f32) -> f32 {
  let t = clamp(x, 0.0, 1.0);
  return t * (2.0 - t);
}

// The dimensional profile of `col` at `hf` of the way up the layer: 0 at
// the cloud's skin and outside it, 1 in its body.
fn cloud_profile(col: CloudColumn, hf: f32) -> f32 {
  if (col.inside <= 0.0 || hf <= 0.0 || hf >= col.top) { return 0.0; }
  let c = frame.clouds;
  let girth = c[12].x * col.height;
  let out = 1.0 - col.inside;
  var rise = 0.0;
  if (hf < girth) {
    let under = c[14].y * (girth - hf) / max(girth, 1e-3);
    rise = under * under;
  } else {
    rise = pow((hf - girth) / max(col.height - girth, 1e-3), c[12].z);
  }
  return cloud_base(col, hf) * cloud_ease((1.0 - sqrt(out * out + rise)) / CLOUD_SIDE);
}

fn cloud_base(col: CloudColumn, hf: f32) -> f32 {
  return cloud_ease(hf / max(col.base, 1e-3));
}

// Wind-aligned gentle altitude lean (metres of horizontal shear over the
// layer's height) instead of a fixed 20° diagonal artifact.
fn cloud_wind_lean(up: f32) -> vec2<f32> {
  let a = frame.clouds[11].w;
  return vec2<f32>(cos(a), sin(a)) * (up * 0.08);
}

// The billows' erosion at the field's point `at`, `up` of the way up its
// own cloud (cloud_up), read over `texel` metres.
fn cloud_coarse(at: vec3<f32>, up: f32, texel: f32) -> f32 {
  let c = frame.clouds;
  let lean = cloud_wind_lean(up);
  // Isotropic sampling in cloud space (no permanent diagonal shear), gently
  // leaned along the real wind heading with altitude.
  let s = vec3<f32>(at.x / c[1].w - lean.x, at.z / c[1].w - lean.y, at.y / c[1].w);
  let q = s * 4.0;
  let mip = clamp(log2(max(texel / c[1].w * 512.0, 1.0)), 0.0, 2.5);
  let n = textureSampleLevel(cloud_noise, cloud_sampler, q, mip);
  let wisps = clamp((n.a - 0.35) / 0.4, 0.0, 1.0);
  let rise = smoothstep(0.0, 0.32, up);
  return c[13].z * mix(1.25 * wisps, n.r, rise) * (1.0 - cloud_detail_share());
}

fn cloud_up(col: CloudColumn, hf: f32) -> f32 {
  return hf / max(col.height, 1e-3);
}

fn cloud_detail_share() -> f32 {
  return 0.32 + 0.32 * clamp((frame.clouds[2].w - 0.35) / 0.65, 0.0, 1.0);
}

// The cloud at the field's point `at`, `hf` of the way up the layer in the
// column `col`, of the dimensional profile `profile` there:
//   x: extinction per metre
//   y: depth under the surface (0 at skin, 1 in body)
//   z: clearance margin under coarse erosion when empty (for cloud_clear)
fn cloud_density_from(at: vec3<f32>, col: CloudColumn, hf: f32, profile: f32, texel: f32, fine: bool) -> vec3<f32> {
  let c = frame.clouds;
  let up = cloud_up(col, hf);
  let coarse = cloud_coarse(at, up, texel);
  if (profile <= coarse) { return vec3<f32>(0.0, 0.0, coarse - profile); }
  let share = c[13].z * cloud_detail_share();
  // Height-aware edge softness: slightly crisper at the condensation base,
  // fluffier on towering billow tops (Nubis² dimensional profile).
  let edge = mix(0.14, 0.24, smoothstep(0.35, 1.0, up));
  var small = 0.5;
  let most = coarse + share;
  // Smooth distance fade for detail erosion (no hard pop or mip washout).
  let detail_ratio = texel * 128.0 / c[1].w;
  let detail_fade = 1.0 - smoothstep(0.45, 1.6, detail_ratio);
  if (fine && detail_fade > 0.01 && profile - most < edge * (1.0 - most)) {
    let lean = cloud_wind_lean(up);
    let dq = vec3<f32>(at.x / c[1].w - lean.x, at.z / c[1].w - lean.y, at.y / c[1].w) * 32.0;
    let dmip = clamp(log2(max(texel / c[1].w * 2048.0, 1.0)), 0.0, 1.75);
    let d = textureSampleLevel(cloud_detail, cloud_sampler, dq, dmip);
    let web = clamp((d.b - 0.24) / 0.49, 0.0, 1.0);
    // Curl-warped billows (d.g) dominate wispy flanks & mid-heights; crisp
    // cauliflower billows (d.r) define upper domes; soft web erodes bases.
    let billow = mix(d.g, d.r, smoothstep(0.25, 0.85, up));
    let raw_small = mix(web, billow, smoothstep(0.0, 0.32, up));
    small = mix(0.5, raw_small, detail_fade);
  }
  let noise = coarse + share * small;
  let deep = clamp((profile - noise) / max(1.0 - noise, 1e-4), 0.0, 1.0);
  // Nubis density-height gradient: bases are softly wispy and thicken upward
  // into dense sunlit domes.
  let height_density = mix(0.48, 1.0, smoothstep(0.0, 0.35, up));
  let density = smoothstep(0.0, edge, deep) * c[0].y * cloud_thin(col) * height_density;
  return vec3<f32>(density, deep, 0.0);
}

const CLOUD_SLOPE: f32 = 60.0;

fn cloud_clear(under: f32) -> f32 {
  let c = frame.clouds;
  return under * c[1].w / (CLOUD_SLOPE * max(c[13].z * (1.0 - cloud_detail_share()), 1e-3));
}

fn cloud_density_in(at: vec3<f32>, hf: f32, texel: f32, fine: bool) -> f32 {
  if (hf <= 0.0 || hf >= 1.0) { return 0.0; }
  let col = cloud_column(cloud_map_at(at, true), hf);
  let profile = cloud_profile(col, hf);
  if (profile <= 0.0) { return 0.0; }
  return cloud_density_from(at, col, hf, profile, texel, fine).x;
}

fn cloud_edge_in(at: vec3<f32>, hf: f32, texel: f32) -> vec3<f32> {
  let col = cloud_column(cloud_map_at(at, true), hf);
  let profile = cloud_profile(col, hf);
  let up = cloud_up(col, hf);
  let fine = 0.5 * frame.clouds[13].z * cloud_detail_share();
  let noise = vec2<f32>(cloud_coarse(at, up, texel), 0.5 * frame.clouds[13].z * (1.0 - cloud_detail_share())) + fine;
  let height_density = mix(0.48, 1.0, smoothstep(0.0, 0.35, up));
  return vec3<f32>((profile - noise) / (max(1.0 - noise, vec2<f32>(1e-3)) * CLOUD_EDGE), cloud_thin(col) * height_density);
}

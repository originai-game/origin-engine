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
//     and the creases between them cut into it, and the density rises to
//     the cloud's own within a few tens of metres of that surface
//     (cloud_density_from).
// frame.clouds: [0] coverage, density, base, thickness; [1] the wind's
// offset, the scale; [2].w detail; [10].xy the viewer's place; [12] the
// kind's envelope: the share of a cloud's height at which it is widest,
// how far up the layer it stands in its core, the power its top rounds off
// by (2: a dome; more: a tower with a flat top; less: a sheet), the
// anvil's reach; [13]
// the kind's field (0 the map's r, 1 its b), the profile's width in the
// field, how much the noise erodes, how much the tops vary; [14] x the
// share of a cloud's height its base takes to become whole (how ragged it
// hangs), y how much it narrows from its widest down to its base. ([14].zw:
// the high layer's place, sky.wgsl.)

// The shape volume and its repeating sampler, the detail volume and the
// map (clouds.rs, clouds_noise.wgsl).
@group(1) @binding(1) var cloud_noise: texture_3d<f32>;
@group(1) @binding(2) var cloud_sampler: sampler;
@group(1) @binding(7) var cloud_detail: texture_3d<f32>;
@group(1) @binding(8) var cloud_map: texture_2d<f32>;

// How far past the surface (in the eroded profile, 0 to 1) the density
// reaches the cloud's own: a cumulus has an edge some tens of metres deep,
// not a haze that thickens to its middle.
const CLOUD_EDGE: f32 = 0.08;

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
// Read between its texels in straight lines, a map of 70 m texels left a
// crease along every texel's edge in the clouds' shape and in their shade,
// and a base seen at a low angle showed them as a ruling of fine lines.
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
// the light's and the shadow bake's, so they are of one cloud (and what
// places clouds in the map is joined here). Smooth between its texels
// (cloud_cubic); not `exact`, in straight lines between them, a quarter
// of the reads, for a march's stride over clear air and its samples behind
// a cloud's visible surface, which never show the creases that leaves.
fn cloud_map_at(at: vec3<f32>, exact: bool) -> vec4<f32> {
  let uv = at.xz / (4.0 * frame.clouds[1].w);
  if (!exact) { return textureSampleLevel(cloud_map, cloud_sampler, uv, 0.0); }
  return cloud_cubic(cloud_map, uv);
}

// How much of the map's field stands in cloud for a cover of `c0`, the
// profile `edge` wide: the field is uniform, the noise erodes the outer
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
// layer's top.
fn cloud_column(m: vec4<f32>, hf: f32) -> CloudColumn {
  let c = frame.clouds;
  let tops = c[12];
  let kind = c[13];
  let field = mix(m.r, m.b, kind.x);
  let reach = tops.w * m.a * smoothstep(0.6, 0.9, hf);
  let inside = clamp((field - 1.0 + cloud_stand(c[0].x, kind.y)) / kind.y + reach, 0.0, 1.0);
  let height = tops.y * mix(1.0, m.g, kind.w);
  let girth = tops.x * height;
  let out = 1.0 - inside;
  let top = girth + (height - girth) * pow(max(1.0 - out * out, 0.0), 1.0 / tops.z);
  // The base hangs lower in some places than in others, by the map's
  // other field (the one that does not place the kind's clouds): 0.4 to
  // 1.6 of the kind's share, so an underside is no plane.
  let base = c[14].x * height * (0.4 + 1.2 * mix(m.b, m.r, kind.x));
  return CloudColumn(inside, height, top, base);
}

// How far in from a cloud's edge its columns hold the cloud's own density
// (of `inside`).
const CLOUD_MARGIN: f32 = 0.3;

// How much of the cloud's own density the column `col` holds: a cloud is
// thin in its outermost columns, the margin of its base, and whole in from
// there (Schneider 2022: the density falls with the dimensional profile
// towards a cloud's edge). As dense at its margin as in its core, a base
// ended in a hard rim, and a column that only just stood in the cloud was
// a small dark rag alone in the sky.
fn cloud_thin(col: CloudColumn) -> f32 {
  return smoothstep(0.0, CLOUD_MARGIN, col.inside);
}

// How far in from a cloud's side the profile is whole (of `inside`).
const CLOUD_SIDE: f32 = 0.7;

// `x` eased up to 1: rising from 0 at twice the slope and levelling off
// where it meets 1, so a ramp made of it ends without a crease (a ramp cut
// off at 1 drew a line round every cloud where its profile became whole).
fn cloud_ease(x: f32) -> f32 {
  let t = clamp(x, 0.0, 1.0);
  return t * (2.0 - t);
}

// The dimensional profile of `col` at `hf` of the way up the layer: 0 at
// the cloud's skin and outside it, 1 in its body. The skin is a round
// envelope (Schneider 2023 makes the profile from a cloud's signed
// distance field; this is that of a dome): how far out of the cloud's
// core the column stands and how far the height is from the cloud's
// widest towards its top are the two axes of an ellipse, so the cloud
// narrows as it rises and its sides lean in to a round top (the product
// of a side's ramp and a top's, the side's was the same at every height:
// a wall). Under its widest the cloud narrows a little to its base, which
// is flat, at the condensation level; the profile becomes whole over a
// share of the cloud's height above it, which the noise erodes: a base
// hangs ragged by some tens of metres, not cut on a plane.
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

// How whole the cloud of `col` is over its base at `hf` of the way up the
// layer: 0 at the layer's base, 1 from the column's own share up.
fn cloud_base(col: CloudColumn, hf: f32) -> f32 {
  return cloud_ease(hf / max(col.base, 1e-3));
}

// The billows' erosion at the field's point `at`, `up` of the way up its
// own cloud (cloud_up), read over `texel` metres: what the profile there
// must pass for the point to hold any cloud, whatever the detail
// (cloud_density_from). Over a cloud's lowest third the billows (their
// creases cut in, their middles stand out) give way to wisps: the
// inverted Worley of another lattice, which erodes the middles of its
// cells and leaves the web between them (Schneider 2015 and 2022: wispy
// under, billowy over; of smooth distances, so the web's crests are
// round: clouds_noise.wgsl). Two fields, blended: the one field and its own
// inverse cancelled halfway, and every cloud had a smooth level sheet
// through it there, which drew bands across the bases.
fn cloud_coarse(at: vec3<f32>, up: f32, texel: f32) -> f32 {
  let c = frame.clouds;
  let s = at.xzy / c[1].w;
  // Sheared, so a tile of the volume (half the scale) shows another
  // slice of it at each place: the shear is whole tiles over the field's
  // period.
  let q = vec3<f32>(s.x, s.y, s.z + s.x * 0.375 + s.y * 0.625) * 2.0;
  // `texel` is a pixel's width; filter over its radius. Height alone
  // must not erase resolved billows from a close view of the cloud top.
  let n = textureSampleLevel(cloud_noise, cloud_sampler, q, log2(max(texel / c[1].w * 128.0, 1.0)));
  // (The field's fiftieth quantiles: 0.35 to 0.75.)
  let wisps = clamp((n.a - 0.35) / 0.4, 0.0, 1.0);
  // (The wisps erode a third deeper than the billows: a base hangs in
  // rags, where the billows' depth left it level.)
  let rise = smoothstep(0.0, 0.3, up);
  return cloud_erosion(up) * mix(1.3 * wisps, n.r, rise) * (1.0 - cloud_detail_share());
}

// How far up its own cloud a point `hf` of the way up the layer is, in
// the column `col`.
fn cloud_up(col: CloudColumn, hf: f32) -> f32 {
  return hf / max(col.height, 1e-3);
}

// The detail's share of the erosion: a third, to three fifths at
// Clouds.detail 1 (wispy).
fn cloud_detail_share() -> f32 {
  return 0.3 + 0.3 * clamp((frame.clouds[2].w - 0.35) / 0.65, 0.0, 1.0);
}

// An anvil spreads at the inversion instead of carrying its tower's
// turbulent erosion unchanged over the cap. Blend by envelope power,
// from a dome (2) to an anvil (5), and by height within that envelope.
fn cloud_erosion(up: f32) -> f32 {
  let c = frame.clouds;
  let cap = clamp((c[12].z - 2.0) / 3.0, 0.0, 1.0) * smoothstep(0.5, 0.9, up);
  return c[13].z * mix(1.0, 0.85, cap);
}

// The cloud at the field's point `at`, `hf` of the way up the layer in the
// column `col`, of the dimensional profile `profile` there: x its
// extinction per metre, y how deep under the surface the point is (0 at
// it, towards 1 in the body), z how far under the billows' erosion the profile is where they
// leave no cloud (cloud_clear).
// `texel`: the size in metres of what the sample stands for (a pixel at
// its distance, a step of the light's): the noise is read at the level
// that holds it. `fine`: with the detail volume; without, the detail at
// its mean (the light's far samples and the shadow's, which are wider than
// it).
//
// The detail erodes as the billows do (cloud_coarse), sixteen times finer:
// its web under, its billows over, curled between. Fade unresolved detail
// to its mean only as the footprint reaches the last volume mip.
fn cloud_density_from(at: vec3<f32>, col: CloudColumn, hf: f32, profile: f32, texel: f32, fine: bool) -> vec3<f32> {
  let c = frame.clouds;
  let up = cloud_up(col, hf);
  let coarse = cloud_coarse(at, up, texel);
  // (Not worth the detail's read where the profile cannot pass the noise
  // with it, or passes it by the whole edge without it.)
  if (profile <= coarse) { return vec3<f32>(0.0, 0.0, coarse - profile); }
  let share = cloud_erosion(up) * cloud_detail_share();
  var small = 0.5;
  let most = coarse + share;
  let lod = log2(max(texel / c[1].w * 1024.0, 1.0));
  let resolved = 1.0 - smoothstep(1.0, 3.0, lod);
  if (fine && resolved > 0.0 && profile - most < CLOUD_EDGE * (1.0 - most)) {
    let d = textureSampleLevel(cloud_detail, cloud_sampler, at.xzy / c[1].w * 32.0, lod);
    // (The web's fiftieth quantiles: 0.23 to 0.73.)
    let web = clamp((d.b - 0.24) / 0.49, 0.0, 1.0);
    small = mix(web, mix(d.g, d.r, clamp(up * 1.5, 0.0, 1.0)), smoothstep(0.0, 0.3, up));
    small = mix(0.5, small, resolved);
  }
  let noise = coarse + share * small;
  let deep = clamp((profile - noise) / (1.0 - noise), 0.0, 1.0);
  return vec3<f32>(smoothstep(0.0, CLOUD_EDGE, deep) * c[0].y * cloud_thin(col), deep, 0.0);
}

// The steepest the billows' erosion (cloud_coarse) changes over a metre,
// halved: the shape volume's fields change by at most some 50 over a tile
// (the sum of their octaves' slopes, each one per cell over the range the
// field is spread from; a quarter of the scale to a tile), and where a
// path meets the steepest of every octave at once is rare.
const CLOUD_SLOPE: f32 = 60.0;

// How many metres from the field's point no cloud can stand, where the
// profile is `under` below the billows' erosion (cloud_density_from's z):
// the erosion cannot fall to the profile nearer than its slope lets it
// (the detail only erodes more).
fn cloud_clear(under: f32) -> f32 {
  let c = frame.clouds;
  return under * c[1].w / (CLOUD_SLOPE * max(c[13].z * (1.0 - cloud_detail_share()), 1e-3));
}

// The extinction per metre at the field's point `at`, `hf` of the way up
// the layer.
fn cloud_density_in(at: vec3<f32>, hf: f32, texel: f32, fine: bool) -> f32 {
  if (hf <= 0.0 || hf >= 1.0) { return 0.0; }
  let col = cloud_column(cloud_map_at(at, true), hf);
  let profile = cloud_profile(col, hf);
  if (profile <= 0.0) { return 0.0; }
  return cloud_density_from(at, col, hf, profile, texel, fine).x;
}

// How far past the cloud's surface the field's point `at` is, `hf` of the
// way up the layer, in the edge's depth (CLOUD_EDGE): under 0 outside the
// cloud, 1 and over where the density is the body's own; without the
// detail. It crosses the surface evenly, where the density starts from
// nothing within metres: what the clouds' shadow is baked from
// (cloud_shadow_bake.wgsl), which reads the cloud every some tens of
// metres. x: of the cloud as it is drawn; y: of the cloud without its
// billows either (the noise at its mean), its envelope; z: how much of the
// cloud's density the column holds (cloud_thin).
fn cloud_edge_in(at: vec3<f32>, hf: f32, texel: f32) -> vec3<f32> {
  let col = cloud_column(cloud_map_at(at, true), hf);
  let profile = cloud_profile(col, hf);
  let up = cloud_up(col, hf);
  let erosion = cloud_erosion(up);
  let fine = 0.5 * erosion * cloud_detail_share();
  let noise = vec2<f32>(cloud_coarse(at, up, texel), 0.5 * erosion * (1.0 - cloud_detail_share())) + fine;
  return vec3<f32>((profile - noise) / (max(1.0 - noise, vec2<f32>(1e-3)) * CLOUD_EDGE), cloud_thin(col));
}

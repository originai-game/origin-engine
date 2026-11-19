// The aurora's footprint (aurora.rs): per texel of a square about the
// viewer, the electron flux into the curtains and the distance to the
// nearest one, which the sky's march (sky.wgsl aurora_over) steps by
// (Lawlor & Genetti 2011, sections 2 and 3.2).
//
// The square is warped radially: texel coordinate q in [-1, 1]² is the
// point q |q| R about the viewer (R the footprint's radius, km), so the
// texels are finest overhead and widest at the horizon, where a curtain
// is far and foreshortened.
//
// Out, per texel: r the sheet's flux (the Kelvin–Helmholtz strip wrapped
// along the nearest curve of each curtain, summed over curtains), g the
// distance to the nearest sheet (km, never more than the true one), b the
// flux-weighted characteristic energy (keV), a the red line's flux, the
// sheet's spread across tens of km (O(1D) lives 110 s and drifts).
//
// With it, the light the whole display sends the clouds under it (`light`,
// into the texture's last row): per line, the cosine-weighted mean radiance
// from above, and round the horizon from the side, as the clouds' clear
// sky light holds the sky's (sky_look.wgsl clear_sky_light). A thin
// emitting shell of column J (the line's emission summed up the column)
// is seen at a radiance J / cos of its tilt to the ray, and a patch dA of
// it subtends dA cos / r², so it sends J dA / r² along the ray, times the
// ray's cosine to the normal. Summed over every texel in a fixed order:
// the same light on every run.

struct Params {
  head: vec4<f32>,  // x side, y radius (km), z curtains, w points per curtain
  look: vec4<f32>,  // x the strip's snapshot (0..3), y the sheet's half-width (km), z the red spread (km), w its scale
  curtain: array<vec4<f32>, 6>,  // x characteristic energy (keV), y the rays' seed, z their contrast
  sky: vec4<f32>,  // x the planet's radius (km), y the viewer's height (km), z the light's row
  columns: array<vec4<f32>, 16>,  // per table row: the lines' columns (km), then their heights (km)
  points: array<vec4<f32>, 768>,  // xy about the viewer (km), z the strip along (periods), w brightness
};

@group(0) @binding(0) var<uniform> params: Params;
@group(0) @binding(1) var strip: texture_2d<f32>;
@group(0) @binding(2) var strip_sampler: sampler;
@group(0) @binding(3) var footprint: texture_storage_2d<rgba16float, write>;
// Per workgroup of `main`: its texels' light from above (xyz, per line),
// then from the side.
@group(0) @binding(4) var<storage, read_write> partials: array<vec4<f32>>;

const MAX_NEAR: u32 = 128u;

var<workgroup> near: array<u32, 128>;
var<workgroup> near_count: atomic<u32>;
var<workgroup> tile_min: atomic<u32>;
// The light's sums, one pair per invocation (`light_sum`).
var<workgroup> light_up: array<vec4<f32>, 256>;
var<workgroup> light_side: array<vec4<f32>, 256>;

fn place(q: vec2<f32>) -> vec2<f32> {
  return q * length(q) * params.head.y;
}

// The distance from `x` to the segment a–b.
fn segment_distance(x: vec2<f32>, a: vec2<f32>, b: vec2<f32>) -> f32 {
  let ab = b - a;
  let h = clamp(dot(x - a, ab) / max(dot(ab, ab), 1e-6), 0.0, 1.0);
  return length(x - a - ab * h);
}

fn hash(a: u32, b: u32) -> f32 {
  var h = a * 747796405u + b * 2891336453u + 0x9e3779b9u;
  h = ((h >> ((h >> 28u) + 4u)) ^ h) * 277803737u;
  h = (h >> 22u) ^ h;
  return f32(h & 0xffffffu) / 16777216.0;
}

// The arc's rays at `a` km along it, its mean 1: field-aligned beams two
// to five kilometres wide (the fine structure of discrete arcs: Hallinan &
// Davis, Planet. Space Sci. 18, 1970), one in a 5 km cell where they
// bunch, few where they thin (a 40 km clumping), with gaps of the arc's
// diffuse glow between. Each beam is a Gaussian widened by `blur` (the
// texel's extent along the arc) with its column kept: the field is
// filtered exactly, never aliasing into a comb; far off, where the
// texels are wider than the beams, it fades to its mean.
fn rays(a: f32, blur: f32, seed: u32) -> f32 {
  let cell = 5.0;
  let b = min(blur, 3.0);
  let i0 = i32(floor(a / cell));
  var sum = 0.0;
  for (var k = -2; k <= 2; k++) {
    let i = i0 + k;
    let j = bitcast<u32>(i);
    let g = f32(i) / 8.0;
    let g0 = floor(g);
    let f = smoothstep(0.0, 1.0, g - g0);
    let clump = mix(hash(bitcast<u32>(i32(g0)), seed ^ 0x51u), hash(bitcast<u32>(i32(g0) + 1), seed ^ 0x51u), f);
    if (hash(j, seed) > mix(0.1, 0.95, clump)) { continue; }
    let at = (f32(i) + hash(j, seed ^ 0x2au)) * cell;
    let sigma = 0.8 + 1.2 * hash(j, seed ^ 0x3bu);
    let h = hash(j, seed ^ 0x4cu);
    let bright = 0.25 + 2.0 * h * h;
    let s2 = sigma * sigma + b * b;
    sum += bright * sigma * inverseSqrt(s2) * exp(-0.5 * (a - at) * (a - at) / s2);
  }
  // The mean: presence 0.525, brightness 0.917, column sqrt(2 pi) 1.4 per 5 km.
  let mean = 0.525 * 0.917 * 2.5066 * 1.4 / cell;
  return mix(sum / mean, 1.0, smoothstep(3.0, 8.0, blur));
}

@compute @workgroup_size(8, 8)
fn main(
  @builtin(global_invocation_id) gid: vec3<u32>,
  @builtin(local_invocation_index) li: u32,
  @builtin(workgroup_id) wid: vec3<u32>,
) {
  let n = u32(params.head.x);
  let per = u32(params.head.w);
  let curtains = u32(params.head.z);
  let hw = params.look.y;
  let red_sigma = params.look.z;
  // A segment farther than this from every texel adds nothing to it.
  let reach = max(hw, 3.0 * red_sigma);
  if (li == 0u) {
    atomicStore(&near_count, 0u);
    atomicStore(&tile_min, bitcast<u32>(1.0e9));
  }
  workgroupBarrier();
  // The tile's bounding circle in the world: its centre's place and the
  // farthest of its corners and edge middles (the warp bends the edges a
  // little; the margin covers it).
  let to_q = 2.0 / f32(n);
  let t0 = vec2<f32>(wid.xy * 8u) * to_q - 1.0;
  let t1 = t0 + 8.0 * to_q;
  let tc = place((t0 + t1) * 0.5);
  var radius = 0.0;
  for (var k = 0u; k < 9u; k++) {
    let f = vec2<f32>(f32(k % 3u), f32(k / 3u)) * 0.5;
    radius = max(radius, length(place(mix(t0, t1, f)) - tc));
  }
  radius = radius * 1.05 + 0.1;
  let segments = curtains * (per - 1u);
  for (var j = li; j < segments; j += 64u) {
    let i = (j / (per - 1u)) * per + j % (per - 1u);
    let d = segment_distance(tc, params.points[i].xy, params.points[i + 1u].xy) - radius;
    atomicMin(&tile_min, bitcast<u32>(max(d, 0.0)));
    if (d < reach) {
      let s = atomicAdd(&near_count, 1u);
      if (s < MAX_NEAR) { near[s] = j; }
    }
  }
  workgroupBarrier();
  let x = place((vec2<f32>(gid.xy) + 0.5) * to_q - 1.0);
  let count = atomicLoad(&near_count);
  // Every segment off the list is at least `reach` away (when the list
  // holds them all); the tile's bound holds regardless.
  var nearest = bitcast<f32>(atomicLoad(&tile_min));
  if (count <= MAX_NEAR) { nearest = max(nearest, reach); }
  // Per curtain, its nearest segment: the signed distance across, the
  // strip's coordinate along, the brightness.
  var best = array<vec4<f32>, 6>();
  var best_j = array<u32, 6>();
  for (var c = 0u; c < 6u; c++) {
    best[c] = vec4<f32>(1.0e9, 0.0, 0.0, 0.0);
    best_j[c] = segments;
  }
  // Atomic insertion orders the near list arbitrarily. On overflow its
  // retained subset is arbitrary too: scan every segment instead.
  let candidates = select(segments, count, count <= MAX_NEAR);
  for (var k = 0u; k < candidates; k++) {
    var j = k;
    if (count <= MAX_NEAR) { j = near[k]; }
    let c = j / (per - 1u);
    let i = c * per + j % (per - 1u);
    let a = params.points[i];
    let b = params.points[i + 1u];
    let ab = b.xy - a.xy;
    let h = clamp(dot(x - a.xy, ab) / max(dot(ab, ab), 1e-6), 0.0, 1.0);
    let off = x - a.xy - ab * h;
    let dist = length(off);
    nearest = min(nearest, dist);
    // At a shared endpoint both segments can be equally near but have
    // different tangents. Always choose the lower index, not the first
    // thread that happened to insert its segment.
    if (dist < abs(best[c].x) || (dist == abs(best[c].x) && j < best_j[c])) {
      let side = select(-1.0, 1.0, ab.x * off.y - ab.y * off.x >= 0.0);
      best[c] = vec4<f32>(dist * side, mix(a.z, b.z, h), mix(a.w, b.w, h), atan2(ab.y, ab.x));
      best_j[c] = j;
    }
  }
  // The texel's extent in the world (km): radially 2|q|R per texel of the
  // warp, round the viewer |q|R. The strip is read over it (gradients for
  // its mips): a sheet thinner than the texel then adds its column to the
  // texel it crosses, wherever it crosses it.
  let qc = (vec2<f32>(gid.xy) + 0.5) * to_q - 1.0;
  let radial = normalize(x + vec2<f32>(1e-6, 0.0));
  let e_r = radial * (2.0 * length(qc) * params.head.y * to_q);
  let e_t = vec2<f32>(-radial.y, radial.x) * (length(qc) * params.head.y * to_q);
  // The strip's period along the curtain (km): its cells are square
  // (aurora.rs: 2 hw times STRIP_X / STRIP_Y).
  let period = hw * 2.0 * 8.0;
  // The strip's snapshots blended by activity: r smooth .. a turbulent.
  let snap = params.look.x;
  let weights = max(vec4<f32>(1.0) - abs(vec4<f32>(snap) - vec4<f32>(0.0, 1.0, 2.0, 3.0)), vec4<f32>(0.0));
  var flux = 0.0;
  var energy = 0.0;
  var weight = 0.0;
  var red = 0.0;
  for (var c = 0u; c < curtains; c++) {
    let s = best[c].x;
    if (abs(s) >= reach) { continue; }
    if (abs(s) < hw) {
      let across = s / hw * 0.5 + 0.5;
      let t = vec2<f32>(cos(best[c].w), sin(best[c].w));
      let side_dir = vec2<f32>(-t.y, t.x);
      let k = vec2<f32>(1.0 / period, 0.5 / hw);
      let gx = vec2<f32>(dot(e_r, t), dot(e_r, side_dir)) * k;
      let gy = vec2<f32>(dot(e_t, t), dot(e_t, side_dir)) * k;
      let dye = dot(textureSampleGrad(strip, strip_sampler, vec2<f32>(best[c].y, across), gx, gy), weights);
      // The texel's extent along the arc, as a Gaussian's sigma (km).
      let blur = 0.3 * length(vec2<f32>(dot(e_r, t), dot(e_t, t)));
      let ray = rays(best[c].y * period, blur, u32(params.curtain[c].y));
      let f = dye * best[c].z * mix(1.0, ray, params.curtain[c].z);
      flux += f;
      energy += f * params.curtain[c].x;
      weight += f;
    }
    let r = best[c].z * params.look.w * exp(-s * s / (2.0 * red_sigma * red_sigma));
    red += r;
    energy += r * params.curtain[c].x;
    weight += r;
  }
  // The energy of the curtains lighting the texel, the first's elsewhere.
  let e0 = select(params.curtain[0].x, energy / max(weight, 1e-9), weight > 1e-6);
  let inside = gid.x < n && gid.y < n;
  if (inside) {
    textureStore(footprint, vec2<i32>(gid.xy), vec4<f32>(flux, max(nearest - hw, 0.0), e0, red));
  }
  // The texel's light on the clouds (the file's head), its area that of
  // its extent in the world.
  let shed = texel_light(flux, red, e0, length(x), length(e_r) * length(e_t));
  light_up[li] = vec4<f32>(shed[0] * f32(inside), 0.0);
  light_side[li] = vec4<f32>(shed[1] * f32(inside), 0.0);
  light_sum(li, 64u);
  if (li == 0u) {
    let g = wid.y * (n / 8u) + wid.x;
    partials[2u * g] = light_up[0];
    partials[2u * g + 1u] = light_side[0];
  }
}

// The light a texel `rho` km out on the ground sends the viewer, per line
// (the file's head): [0] from above, [1] round the horizon from the side,
// each over π. `area` its extent (km²) on the ground.
fn texel_light(flux: f32, red: f32, e0: f32, rho: f32, area: f32) -> array<vec3<f32>, 2> {
  var out = array<vec3<f32>, 2>(vec3<f32>(0.0), vec3<f32>(0.0));
  if (flux + red <= 0.0) { return out; }
  // The table's row by the characteristic energy, as the sky's march reads it.
  let row = clamp(log2(max(e0, 0.01) / 0.5) / 0.67, 0.0, 7.0);
  let r0 = u32(row);
  let r1 = min(r0 + 1u, 7u);
  let column = mix(params.columns[r0], params.columns[r1], row - f32(r0)).xyz;
  let height = mix(params.columns[8u + r0], params.columns[8u + r1], row - f32(r0)).xyz;
  let j = vec3<f32>(flux * column.x, flux * column.y, red * column.z);
  let radius = params.sky.x;
  let eye = radius + params.sky.y;
  let angle = rho / radius;
  for (var k = 0u; k < 3u; k++) {
    let up = (radius + height[k]) * cos(angle) - eye;
    // Under the viewer's horizon it lights nothing.
    if (up <= 0.0) { continue; }
    let across = (radius + height[k]) * sin(angle);
    let r2 = up * up + across * across;
    let r = sqrt(r2);
    // The patch at its height, wider than on the ground.
    let lift = (radius + height[k]) / radius;
    let sent = j[k] * area * lift * lift / r2;
    out[0][k] = sent * (up / r) / PI;
    // Round the horizon a vertical surface's cosine averages 1/π of the
    // ray's horizontal part.
    out[1][k] = sent * (across / r) / (PI * PI);
  }
  return out;
}

// Sums light_up and light_side over the first `count` invocations into
// their first, pairwise in a fixed order.
fn light_sum(li: u32, count: u32) {
  var stride = count / 2u;
  loop {
    workgroupBarrier();
    if (li < stride) {
      light_up[li] += light_up[li + stride];
      light_side[li] += light_side[li + stride];
    }
    if (stride <= 1u) { break; }
    stride /= 2u;
  }
  workgroupBarrier();
}

// The light of the whole footprint, from `main`'s workgroups' shares, into
// the texture's light row: texel 0 from above, 1 from the side (per line).
@compute @workgroup_size(256)
fn light(@builtin(local_invocation_index) li: u32) {
  let groups = arrayLength(&partials) / 2u;
  var up = vec4<f32>(0.0);
  var side = vec4<f32>(0.0);
  for (var g = li; g < groups; g += 256u) {
    up += partials[2u * g];
    side += partials[2u * g + 1u];
  }
  light_up[li] = up;
  light_side[li] = side;
  light_sum(li, 256u);
  if (li == 0u) {
    let row = i32(params.sky.z);
    textureStore(footprint, vec2<i32>(0, row), light_up[0]);
    textureStore(footprint, vec2<i32>(1, row), light_side[0]);
  }
}

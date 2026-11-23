// The clouds' baked fields (clouds.rs `CloudNoise`), each made once by a
// compute pass the first frame that has clouds, each tiling in every axis
// (every lattice point is hashed from its cell wrapped by the frequency):
//
//   shape, 128³ (one tile a quarter of Clouds.scale):
//     r: the billows, Schneider's Perlin-Worley as what erodes a cloud:
//        the distance to the nearest feature point (Worley's F1) at 4, 8,
//        16 and 32 cells, low in a billow's middle and high in the creases
//        between billows, less where a gradient noise of 2, 4 and 8 cells
//        is high (whole lumps some hundreds of metres across stand out of
//        a cloud there, so its billows are not all one size), spread over
//        0..1
//     g: ridged gradient noise at 4, 8 and 16 cells (streaks: the high layer)
//     b: an inverted Worley fBm at 8, 16 and 32 cells (the high layer's body)
//     a: the wisps a base hangs in: that fBm of smooth distances, which
//        have no crease where two cells meet
//   detail, 64³ (one tile a thirty-second of the scale):
//     r: the billows again, eight times finer
//     g: the same field read through a gradient-noise warp (curled wisps)
//     b: an inverted Worley fBm at 8 and 16 cells
//   map, 512² (the whole cloud field: four tiles of the scale):
//     r: where clouds stand: blobs of two sizes, each in its own regions
//        (6 and 12 cells where a value noise of 3 cells is high, 12 and 24
//        where it is low: a field of one lattice drew every cloud the same
//        size), over a broad value noise, equalised (uniform over 0..1, so
//        the share of the map over 1 - c is c)
//     g: how tall a cloud stands there: value noise and the regions of
//        the larger blobs (a wide cloud is a tall one; as tall, a narrow
//        one was a column), equalised
//     b: a field of closer cells (32 and 64) for sheets of small clouds,
//        equalised
//     a: how vigorous the convection is there (which towers spread),
//        equalised value noise
//
// and the mips of the two volumes, each level the box average of the one
// above. What each is read for: clouds_shape.wgsl.
//
// The recipes follow published descriptions (third_party/nubis-clouds.md):
// Schneider's Perlin-Worley shape and Worley detail (2015, 2017) and the
// curled detail of Nubis³ (2023); the code and the fields are Origin's own.

@group(0) @binding(0) var volume_out: texture_storage_3d<rgba8unorm, write>;
@group(0) @binding(1) var map_out: texture_storage_2d<rgba16float, write>;
@group(0) @binding(2) var mip_src: texture_3d<f32>;

fn hash(c: vec3<i32>, salt: u32) -> f32 {
  return lattice3(c, salt * 2654435761u);
}

fn wrap(c: vec3<i32>, f: i32) -> vec3<i32> {
  return ((c % f) + f) % f;
}

// The distance to the nearest feature point of a lattice of `f` cells, in
// cells (Worley's F1), at most 1.
fn cell_distance(p: vec3<f32>, f: f32, salt: u32) -> f32 {
  let q = p * f;
  let cell = vec3<i32>(floor(q));
  let fi = i32(f);
  var d = 1e9;
  for (var z = -1; z <= 1; z++) {
    for (var y = -1; y <= 1; y++) {
      for (var x = -1; x <= 1; x++) {
        let c = cell + vec3<i32>(x, y, z);
        let w = wrap(c, fi);
        let fp = vec3<f32>(c) + vec3<f32>(hash(w, salt), hash(w, salt + 1u), hash(w, salt + 2u));
        let v = q - fp;
        d = min(d, dot(v, v));
      }
    }
  }
  return clamp(sqrt(d), 0.0, 1.0);
}

// The same distance with no crease where two cells meet: the smooth
// minimum of the distances to the feature points (the logarithm of the sum
// of their exponentials, an eighth of a cell wide). What is left between
// eroded cells then has a round crest: with the hard minimum's ridge, the
// light drew a line along every one.
fn cell_distance_smooth(p: vec3<f32>, f: f32, salt: u32) -> f32 {
  let q = p * f;
  let cell = vec3<i32>(floor(q));
  let fi = i32(f);
  var sum = 0.0;
  for (var z = -1; z <= 1; z++) {
    for (var y = -1; y <= 1; y++) {
      for (var x = -1; x <= 1; x++) {
        let c = cell + vec3<i32>(x, y, z);
        let w = wrap(c, fi);
        let fp = vec3<f32>(c) + vec3<f32>(hash(w, salt), hash(w, salt + 1u), hash(w, salt + 2u));
        sum += exp(-8.0 * length(q - fp));
      }
    }
  }
  return clamp(-log(sum) / 8.0, 0.0, 1.0);
}

// Gradient noise (Perlin 2002, the quintic fade), about -0.7 to 0.7.
fn perlin(p: vec3<f32>, f: f32) -> f32 {
  let q = p * f;
  let c = vec3<i32>(floor(q));
  let t = fract(q);
  let s = t * t * t * (t * (t * 6.0 - 15.0) + 10.0);
  let fi = i32(f);
  var v = 0.0;
  for (var k = 0; k < 8; k++) {
    let o = vec3<i32>(k & 1, (k >> 1) & 1, (k >> 2) & 1);
    let w = wrap(c + o, fi);
    let g = vec3<f32>(hash(w, 4u), hash(w, 5u), hash(w, 6u)) * 2.0 - 1.0;
    let m = mix(1.0 - s, s, vec3<f32>(o));
    v += dot(g / max(length(g), 1e-4), t - vec3<f32>(o)) * m.x * m.y * m.z;
  }
  return v;
}

// 1 along the noise's zero crossings, falling to 0 where it is strong.
fn ridged(p: vec3<f32>, f: f32) -> f32 {
  return 1.0 - min(abs(perlin(p, f)) * 2.5, 1.0);
}

// The billows of a volume whose coarsest lattice has `f` cells: F1 at
// three octaves, over the range it takes (measured: its sixteenth
// quantiles run 0.35 to 0.68).
fn billows(p: vec3<f32>, f: f32, salt: u32) -> f32 {
  let d = 0.5 * cell_distance(p, f, salt) + 0.3 * cell_distance(p, f * 2.0, salt + 3u) + 0.2 * cell_distance(p, f * 4.0, salt + 6u);
  return clamp((d - 0.28) / 0.46, 0.0, 1.0);
}

// The shape volume's billows (its r, above).
fn perlin_worley(p: vec3<f32>) -> f32 {
  let d = 0.45 * cell_distance(p, 4.0, 10u) + 0.27 * cell_distance(p, 8.0, 13u) + 0.18 * cell_distance(p, 16.0, 16u) + 0.10 * cell_distance(p, 32.0, 19u);
  // (Measured: its fiftieth quantiles run 0.32 to 0.72.)
  let worley = clamp((d - 0.30) / 0.42, 0.0, 1.0);
  let low = 0.5 * perlin(p, 2.0) + 0.3 * perlin(p, 4.0) + 0.2 * perlin(p, 8.0);
  let lumps = clamp(0.5 + 1.6 * low, 0.0, 1.0);
  // (The product's fiftieth quantiles: 0 to 0.64.)
  return clamp(worley * (1.0 - lumps) / 0.6, 0.0, 1.0);
}

// Value noise over the map.
fn value2(p: vec2<f32>, f: f32, salt: u32) -> f32 {
  let q = p * f;
  let c = vec2<i32>(floor(q));
  let t = fract(q);
  let s = t * t * (3.0 - 2.0 * t);
  let fi = i32(f);
  var v = 0.0;
  for (var k = 0; k < 4; k++) {
    let o = vec2<i32>(k & 1, k >> 1);
    let m = mix(1.0 - s, s, vec2<f32>(o));
    v += hash(wrap(vec3<i32>(c + o, 0), fi), salt) * m.x * m.y;
  }
  return v;
}

// 1 at the feature point of a cell of a lattice of `f` cells over the map,
// falling to 0 a cell away.
fn blob2(p: vec2<f32>, f: f32, salt: u32) -> f32 {
  let q = p * f;
  let cell = vec2<i32>(floor(q));
  let fi = i32(f);
  var d = 1e9;
  for (var y = -1; y <= 1; y++) {
    for (var x = -1; x <= 1; x++) {
      let c = cell + vec2<i32>(x, y);
      let w = wrap(vec3<i32>(c, 0), fi);
      let v = q - (vec2<f32>(c) + vec2<f32>(hash(w, salt), hash(w, salt + 1u)));
      d = min(d, dot(v, v));
    }
  }
  return 1.0 - min(d, 1.0);
}

// The tallest, at the map's point `p`, of the domes of a lattice of `f`
// cells: each cell's a dome of its own height (a half to the whole) about
// its feature point, four fifths of a cell in radius, level where it
// ends. Cells that stand apart: the nearest point's distance (blob2) is a
// field with no ground between its cells, and a cover cut from it joined
// them into banks.
fn dome2(p: vec2<f32>, f: f32, salt: u32) -> f32 {
  let q = p * f;
  let cell = vec2<i32>(floor(q));
  let fi = i32(f);
  var best = 0.0;
  for (var y = -1; y <= 1; y++) {
    for (var x = -1; x <= 1; x++) {
      let c = cell + vec2<i32>(x, y);
      let w = wrap(vec3<i32>(c, 0), fi);
      let v = q - (vec2<f32>(c) + vec2<f32>(hash(w, salt), hash(w, salt + 1u)));
      let s = max(1.0 - dot(v, v) / 0.64, 0.0);
      best = max(best, (0.5 + 0.5 * hash(w, salt + 2u)) * s * sqrt(s));
    }
  }
  return best;
}

// The quantiles of the map's fields before equalising, at sixteenths
// (measured over these recipes).
const FIELD_Q = array<f32, 17>(0.04494, 0.2286, 0.2742, 0.3057, 0.3325, 0.3567, 0.3804, 0.4047, 0.4287, 0.4532, 0.4782, 0.5037, 0.5329, 0.5664, 0.6048, 0.6601, 0.9195);
const TALL_Q = array<f32, 17>(0.1172, 0.2826, 0.3267, 0.3657, 0.4245, 0.5125, 0.5953, 0.6422, 0.6737, 0.7007, 0.7289, 0.7539, 0.7781, 0.8, 0.8253, 0.8494, 0.9335);
const CELLS_Q = array<f32, 17>(0.0, 0.5732, 0.636, 0.6777, 0.7096, 0.7367, 0.7602, 0.7814, 0.8012, 0.8199, 0.8383, 0.8565, 0.875, 0.8945, 0.9161, 0.9422, 0.9999);
const VALUE_Q = array<f32, 17>(0.06609, 0.2428, 0.2955, 0.3295, 0.3558, 0.3803, 0.4055, 0.4303, 0.4584, 0.4869, 0.5165, 0.5443, 0.5781, 0.6165, 0.6599, 0.7239, 0.9787);

// `x` through the cumulative distribution its quantiles `q` sample: a
// monotone cubic through them (Fritsch and Carlson 1980: each tangent the
// harmonic mean of the slopes either side), so the field's slope has no
// step. Straight between the quantiles, it had one along the field's
// contour at every sixteenth: a crease in every cloud's side and top
// there, which the light drew as contour lines across the bases.
fn equalise(x: f32, q: array<f32, 17>) -> f32 {
  if (x <= q[0]) { return 0.0; }
  if (x >= q[16]) { return 1.0; }
  var i = 1;
  for (; i < 16; i++) {
    if (x <= q[i]) { break; }
  }
  let h = max(q[i] - q[i - 1], 1e-5);
  let s = 1.0 / (16.0 * h);
  var m0 = s;
  var m1 = s;
  if (i > 1) {
    let before = 1.0 / (16.0 * max(q[i - 1] - q[i - 2], 1e-5));
    m0 = 2.0 * before * s / (before + s);
  }
  if (i < 16) {
    let after = 1.0 / (16.0 * max(q[i + 1] - q[i], 1e-5));
    m1 = 2.0 * after * s / (after + s);
  }
  let t = (x - q[i - 1]) / h;
  let t2 = t * t;
  let t3 = t2 * t;
  return (2.0 * t3 - 3.0 * t2 + 1.0) * f32(i - 1) / 16.0 + (t3 - 2.0 * t2 + t) * h * m0 + (3.0 * t2 - 2.0 * t3) * f32(i) / 16.0 + (t3 - t2) * h * m1;
}

@compute @workgroup_size(4, 4, 4)
fn shape(@builtin(global_invocation_id) id: vec3<u32>) {
  let n = textureDimensions(volume_out);
  if (any(id >= n)) { return; }
  let p = (vec3<f32>(id) + 0.5) / vec3<f32>(n);
  let streaks = 0.5 * ridged(p, 4.0) + 0.3 * ridged(p, 8.0) + 0.2 * ridged(p, 16.0);
  let w8 = 1.0 - cell_distance(p, 8.0, 30u);
  let w16 = 1.0 - cell_distance(p, 16.0, 33u);
  let w32 = 1.0 - cell_distance(p, 32.0, 36u);
  let wisps = 1.0 - 0.625 * cell_distance_smooth(p, 8.0, 30u) - 0.25 * cell_distance_smooth(p, 16.0, 33u) - 0.125 * cell_distance_smooth(p, 32.0, 36u);
  textureStore(volume_out, vec3<i32>(id), vec4<f32>(
    perlin_worley(p),
    streaks,
    w8 * 0.625 + w16 * 0.25 + w32 * 0.125,
    wisps));
}

@compute @workgroup_size(4, 4, 4)
fn detail(@builtin(global_invocation_id) id: vec3<u32>) {
  let n = textureDimensions(volume_out);
  if (any(id >= n)) { return; }
  let p = (vec3<f32>(id) + 0.5) / vec3<f32>(n);
  // The warp's three components are one tiling noise read at three places.
  let warp = vec3<f32>(perlin(p, 4.0), perlin(p + vec3<f32>(0.31, 0.17, 0.53), 4.0), perlin(p + vec3<f32>(0.71, 0.43, 0.19), 4.0));
  let web = 1.0 - 0.6 * cell_distance(p, 8.0, 60u) - 0.4 * cell_distance(p, 16.0, 63u);
  textureStore(volume_out, vec3<i32>(id), vec4<f32>(
    billows(p, 4.0, 40u),
    billows(p + warp * 0.07, 4.0, 40u),
    web,
    1.0));
}

@compute @workgroup_size(8, 8, 1)
fn map(@builtin(global_invocation_id) id: vec3<u32>) {
  let n = textureDimensions(map_out);
  if (any(id.xy >= n)) { return; }
  let p = (vec2<f32>(id.xy) + 0.5) / vec2<f32>(n);
  // The cover varies by region, a third of the field's period across: a
  // sky has banks of cloud and clear reaches between them (even from
  // place to place, the clouds far off closed into one band along the
  // horizon). Over 4.5 km squares a cover of 0.4 runs from none to nine
  // tenths.
  let region = 0.6 * value2(p, 3.0, 80u) + 0.25 * value2(p, 6.0, 81u) + 0.15 * value2(p, 12.0, 79u);
  let big = smoothstep(0.35, 0.65, value2(p, 3.0, 94u));
  // Cells that stand apart (dome2), of two sizes in each kind of region,
  // fewer and lower where the region's cover is low: the far field is
  // cells with sky between them, where a field with no ground between its
  // cells closed into a bank at any cover.
  let small = max(dome2(p, 12.0, 82u), 0.8 * dome2(p, 24.0, 84u));
  let large = max(dome2(p, 6.0, 96u), 0.8 * dome2(p, 12.0, 98u));
  let field = 0.85 * mix(small, large, big) * (0.4 + 0.6 * smoothstep(0.25, 0.75, region)) + 0.15 * region;
  let cells = 0.6 * blob2(p, 32.0, 86u) + 0.4 * blob2(p, 64.0, 88u);
  let tall = 0.5 * (0.6 * value2(p, 8.0, 90u) + 0.4 * value2(p, 16.0, 91u)) + 0.5 * big;
  let vigour = 0.6 * value2(p, 8.0, 92u) + 0.4 * value2(p, 16.0, 93u);
  textureStore(map_out, vec2<i32>(id.xy), vec4<f32>(
    equalise(field, FIELD_Q),
    equalise(tall, TALL_Q),
    equalise(cells, CELLS_Q),
    equalise(vigour, VALUE_Q)));
}

// A level of a volume from the level above it (`mip_src`, that level alone).
@compute @workgroup_size(4, 4, 4)
fn mips(@builtin(global_invocation_id) id: vec3<u32>) {
  let n = textureDimensions(volume_out);
  if (any(id >= n)) { return; }
  let q = vec3<i32>(id) * 2;
  var sum = vec4<f32>(0.0);
  for (var k = 0; k < 8; k++) {
    sum += textureLoad(mip_src, q + vec3<i32>(k & 1, (k >> 1) & 1, (k >> 2) & 1), 0);
  }
  textureStore(volume_out, vec3<i32>(id), sum * 0.125);
}

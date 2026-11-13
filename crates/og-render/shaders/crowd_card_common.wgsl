// Crowd impostor cards (crowd.rs **Impostors**, crowd_impostor.rs): a
// crowd member far away drawn as one card showing its full mesh baked
// into an atlas of `views` columns (directions around the vertical at the
// bake's elevation) by `frames` rows (frames of its clip). Shared by the
// lit, prepass and shadow passes; follows scatter_common.wgsl (group 2:
// the crowd's sphere, views, frames and elevation) and material.wgsl
// (group 1: the atlas as the card's material).
//
// The atlas (crowd_bake.wgsl), premultiplied by coverage:
//   base_color_tex  rgb: the parts' colour detail, a: coverage
//   normal_tex      xy: the normal in the view's basis (octahedral),
//                   z: the depth towards the viewer in radii, w: the part
// material.custom[p]: part p's colour (rgb) and roughness (a).
//
// group 2 (scatter.rs ScatterGpu): impostor xyz the sphere's centre, w
// its radius; fade.z the views, fade.w the frames; impostor_light.w the
// elevation (radians); impostor_light.xyz towards the sun.

// A card's record (crowd.rs `card_record`): the member's transform, its
// place among the frames (signed: whether it receives shadows) and its
// look's flash, flash colour and tint.
struct CardIn {
  @location(4) m0: vec3<f32>,
  @location(5) m1: vec3<f32>,
  @location(6) m2: vec3<f32>,
  @location(7) m3: vec3<f32>,
  @location(8) look: vec4<f32>,
};

fn card_views() -> u32 { return max(u32(scatter.fade.z), 1u); }
fn card_frames() -> u32 { return max(u32(scatter.fade.w), 1u); }

// The direction column `col` was baked from (from the centre towards the
// viewer) and its right and up (crowd_bake.wgsl bake_basis).
fn card_basis(col: f32) -> mat3x3<f32> {
  let a = 6.283185307 * col / f32(card_views());
  let e = scatter.impostor_light.w;
  let d = vec3<f32>(cos(e) * sin(a), sin(e), cos(e) * cos(a));
  let r = normalize(cross(vec3<f32>(0.0, 1.0, 0.0), d));
  return mat3x3<f32>(r, cross(d, r), d);
}

struct Card {
  world: vec4<f32>,
  // The card's point about the sphere's centre in radii, the model's space.
  local: vec3<f32>,
  // The two columns around the eye's direction and the second's weight.
  cols: vec2<f32>,
  col_w: f32,
  // The two frames around the member's place in its clip and the second's weight.
  rows: vec2<f32>,
  row_w: f32,
  // The sphere's radius in the world, and the card's facing there.
  extent: f32,
  toward: vec3<f32>,
};

// The card's corner `vertex` (0..6) of a member at `model` facing `toward`
// (a point, or a direction when `is_dir`) around the vertical, tilted to
// the bake's elevation; `place`: its record's look.x.
fn card_make(model: mat4x4<f32>, vertex: u32, toward: vec3<f32>, is_dir: bool, place: f32) -> Card {
  var corners = array<vec2<f32>, 6>(
    vec2<f32>(-1.0, -1.0), vec2<f32>(1.0, -1.0), vec2<f32>(1.0, 1.0),
    vec2<f32>(-1.0, -1.0), vec2<f32>(1.0, 1.0), vec2<f32>(-1.0, 1.0),
  );
  let q = corners[vertex % 6u];
  let m3 = mat3x3<f32>(model[0].xyz, model[1].xyz, model[2].xyz);
  let c = scatter.impostor.xyz;
  let radius = scatter.impostor.w;
  let centre = mul_point(model, vec4<f32>(c, 1.0)).xyz;
  let to_eye = select(toward - centre, toward, is_dir);
  let v = transpose(m3) * to_eye;
  let views = f32(card_views());
  let at = atan2(v.x, v.z) / 6.283185307 * views;
  let col = at - views * floor(at / views);
  var o: Card;
  o.cols = vec2<f32>(floor(col), (floor(col) + 1.0) % views);
  o.col_w = fract(col);
  let b = card_basis(col);
  o.local = b[0] * q.x + b[1] * q.y;
  o.world = mul_point(model, vec4<f32>(c + o.local * radius, 1.0));
  let frames = f32(card_frames());
  let place_ = select(-1.0 - place, place, place >= 0.0);
  let row = place_ - frames * floor(place_ / frames);
  o.rows = vec2<f32>(floor(row), (floor(row) + 1.0) % frames);
  o.row_w = fract(row);
  let scale = max(length(model[0].xyz), max(length(model[1].xyz), length(model[2].xyz)));
  o.extent = radius * scale;
  o.toward = normalize(m3 * b[2]);
  return o;
}

// The atlas's level of detail for this pixel (card_set_level), shared by
// every sample.
var<private> card_level: f32;

// Sets card_level from the card's point (radii), first in a fragment
// stage, in uniform control flow.
fn card_set_level(local: vec3<f32>) {
  let size = f32(textureDimensions(base_color_tex, 0).x) / f32(card_views());
  let d = max(length(dpdxFine(local)), length(dpdyFine(local))) * size * 0.5;
  let top = f32(textureNumLevels(base_color_tex) - 1u);
  card_level = clamp(log2(max(d, 1e-6)), 0.0, top);
}

// Where tile (col, row)'s point `uv` ([-1, 1]², y up) is in the atlas,
// kept half a texel of the level read inside the tile.
fn card_atlas(col: f32, row: f32, uv: vec2<f32>) -> vec2<f32> {
  let size = vec2<f32>(textureDimensions(base_color_tex, 0));
  let tiles = vec2<f32>(f32(card_views()), f32(card_frames()));
  let half = 0.5 * exp2(card_level) * tiles / size;
  let inside = clamp(vec2<f32>(uv.x * 0.5 + 0.5, 0.5 - uv.y * 0.5), half, vec2<f32>(1.0) - half);
  return (vec2<f32>(col, row) + inside) / tiles;
}

// The card's surface where the pixel's point is: the atlas's four tiles
// around it (two columns by two frames) blended by weight and coverage.
struct CardSample {
  coverage: f32,
  // The colour detail, the normal (the model's space), the depth towards
  // the viewer (radii) and the part, all divided by the coverage.
  detail: vec3<f32>,
  normal: vec3<f32>,
  depth: f32,
  part: f32,
};

fn card_sample(local: vec3<f32>, cols: vec2<f32>, col_w: f32, rows: vec2<f32>, row_w: f32) -> CardSample {
  var a = vec4<f32>(0.0);
  var d = vec4<f32>(0.0);
  var n = vec3<f32>(0.0);
  for (var i = 0u; i < 2u; i++) {
    let col = cols[i];
    let b = card_basis(col);
    let uv = vec2<f32>(dot(local, b[0]), dot(local, b[1]));
    let inside = select(0.0, 1.0, all(abs(uv) <= vec2<f32>(1.0)));
    let wc = select(1.0 - col_w, col_w, i == 1u) * inside;
    for (var j = 0u; j < 2u; j++) {
      let w = wc * select(1.0 - row_w, row_w, j == 1u);
      let at = card_atlas(col, rows[j], uv);
      let ak = textureSampleLevel(base_color_tex, material_sampler, at, card_level);
      let dk = textureSampleLevel(normal_tex, material_sampler, at, card_level);
      a += ak * w;
      d += dk * w;
      // Each tile's normal out of its own basis (premultiplied: divided
      // by its coverage first).
      n += b * oct_decode(dk.xy / max(ak.a, 1e-4)) * (ak.a * w);
    }
  }
  var o: CardSample;
  o.coverage = a.a;
  let c = max(a.a, 1e-4);
  o.detail = a.rgb / c;
  o.normal = normalize(n + vec3<f32>(0.0, 1e-6, 0.0));
  o.depth = d.z / c;
  o.part = d.w / c;
  return o;
}

// Part `part`'s colour and roughness (a fraction between two parts, where
// a coarse level of the atlas averaged them, blends theirs).
fn card_part(part: f32) -> vec4<f32> {
  let last = max(material.custom[4].x - 1.0, 0.0);
  let p = clamp(part, 0.0, last);
  let lo = u32(floor(p));
  let hi = min(lo + 1u, u32(last));
  return mix(material.custom[lo], material.custom[hi], p - floor(p));
}

// A colour packed 8:8:8 (instances.rs fx_bits; pbr.wgsl look_rgb).
fn card_rgb(packed: f32) -> vec3<f32> {
  let u = u32(packed);
  return vec3<f32>(f32((u >> 16u) & 255u), f32((u >> 8u) & 255u), f32(u & 255u)) / 255.0;
}

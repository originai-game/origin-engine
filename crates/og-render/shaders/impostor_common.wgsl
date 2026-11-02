// Octahedral impostors (impostor.rs): the card, the frames it blends and
// their samples. Shared by the lit, prepass and shadow passes; follows
// scatter_common.wgsl (group 2: the impostor's sphere and grid) and
// material.wgsl (group 1: the atlas as base_color_tex and normal_tex).

fn imp_sign(v: f32) -> f32 { return select(-1.0, 1.0, v >= 0.0); }

// impostor.rs `oct_encode`: +y at the centre, -y at the corners.
fn imp_oct_encode(d: vec3<f32>) -> vec2<f32> {
  let n = d / max(abs(d.x) + abs(d.y) + abs(d.z), 1e-12);
  if (n.y >= 0.0) { return n.xz; }
  return vec2<f32>((1.0 - abs(n.z)) * imp_sign(n.x), (1.0 - abs(n.x)) * imp_sign(n.z));
}

fn imp_oct_decode(p: vec2<f32>) -> vec3<f32> {
  let y = 1.0 - abs(p.x) - abs(p.y);
  var x = p.x;
  var z = p.y;
  if (y < 0.0) {
    x = (1.0 - abs(p.y)) * imp_sign(p.x);
    z = (1.0 - abs(p.x)) * imp_sign(p.y);
  }
  return normalize(vec3<f32>(x, y, z));
}

fn imp_grid() -> u32 { return max(u32(scatter.impostor_grid.x), 2u); }

// Frame `f` (column + row × grid) looks from this direction.
fn imp_frame_dir(f: u32) -> vec3<f32> {
  let g = imp_grid();
  let n = f32(g - 1u);
  return imp_oct_decode(vec2<f32>(f32(f % g), f32(f / g)) / n * 2.0 - 1.0);
}

// impostor.rs `frame_basis`: the right of a view from `d` (up = cross(d, right)).
fn imp_right(d: vec3<f32>) -> vec3<f32> {
  let reference = select(vec3<f32>(0.0, 1.0, 0.0), vec3<f32>(0.0, 0.0, 1.0), abs(d.y) > 0.999);
  return normalize(cross(reference, d));
}

struct ImpCard {
  world: vec4<f32>,
  // Each frame's point in its square ([-1, 1]², y up), where the ray
  // from the eye through the card's vertex meets the frame's plane.
  uv01: vec4<f32>,
  uv2: vec2<f32>,
  frames: vec3<u32>,
  weights: vec3<f32>,
  // The model's sphere radius in the world.
  extent: f32,
};

// Where a frame's plane is met by the ray along -v through the card's
// point `p` (both about the centre, the model's space), in the frame's square.
fn imp_frame_uv(p: vec3<f32>, v: vec3<f32>, f: u32) -> vec2<f32> {
  let d = imp_frame_dir(f);
  let r = imp_right(d);
  let u = cross(d, r);
  let x = p - v * (dot(p, d) / max(dot(v, d), 0.05));
  return vec2<f32>(dot(x, r), dot(x, u)) / scatter.impostor.w;
}

// The card's corner `vertex` (0..6, two triangles) of the copy `model`
// facing `toward` (a point, or a direction when `is_dir`).
fn imp_card(model: mat4x4<f32>, vertex: u32, toward: vec3<f32>, is_dir: bool) -> ImpCard {
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
  // The eye's direction in the model's space (its scale divides out).
  let v = normalize(transpose(m3) * to_eye);
  let r = imp_right(v);
  let u = cross(v, r);
  let p = (r * q.x + u * q.y) * radius;
  var o: ImpCard;
  o.world = mul_point(model, vec4<f32>(c + p, 1.0));
  // The three frames around v (impostor.rs `blend_frames`).
  let g = imp_grid();
  let n = f32(g - 1u);
  let at = clamp((imp_oct_encode(v) * 0.5 + 0.5) * n, vec2<f32>(0.0), vec2<f32>(n));
  let cell = min(floor(at), vec2<f32>(n - 1.0));
  let f = at - cell;
  let base = u32(cell.x) + u32(cell.y) * g;
  if (f.x + f.y <= 1.0) {
    o.frames = vec3<u32>(base, base + 1u, base + g);
    o.weights = vec3<f32>(1.0 - f.x - f.y, f.x, f.y);
  } else {
    o.frames = vec3<u32>(base + g + 1u, base + g, base + 1u);
    o.weights = vec3<f32>(f.x + f.y - 1.0, 1.0 - f.x, 1.0 - f.y);
  }
  o.uv01 = vec4<f32>(imp_frame_uv(p, v, o.frames.x), imp_frame_uv(p, v, o.frames.y));
  o.uv2 = imp_frame_uv(p, v, o.frames.z);
  o.extent = radius * max(length(model[0].xyz), max(length(model[1].xyz), length(model[2].xyz)));
  return o;
}

// Where frame `f`'s point `uv` is in the atlas, kept half a texel inside
// the frame's square; `w` is zeroed outside it.
fn imp_atlas(f: u32, uv: vec2<f32>) -> vec2<f32> {
  let g = imp_grid();
  let size = vec2<f32>(textureDimensions(base_color_tex, 0));
  let half = 0.5 * f32(g) / size;
  let inside = clamp(vec2<f32>(uv.x * 0.5 + 0.5, 0.5 - uv.y * 0.5), half, vec2<f32>(1.0) - half);
  return (vec2<f32>(f32(f % g), f32(f / g)) + inside) / f32(g);
}

fn imp_inside(uv: vec2<f32>) -> f32 {
  return select(0.0, 1.0, all(abs(uv) <= vec2<f32>(1.0)));
}

// The three frames' coverage by weight (the blend's alpha), each also
// returned scaled into `wa`.
struct ImpCoverage {
  wa: vec3<f32>,
  alpha: f32,
  a0: vec2<f32>,
  a1: vec2<f32>,
  a2: vec2<f32>,
};

fn imp_coverage(uv01: vec4<f32>, uv2: vec2<f32>, frames: vec3<u32>, weights: vec3<f32>) -> ImpCoverage {
  var o: ImpCoverage;
  o.a0 = imp_atlas(frames.x, uv01.xy);
  o.a1 = imp_atlas(frames.y, uv01.zw);
  o.a2 = imp_atlas(frames.z, uv2);
  let c = vec3<f32>(
    textureSample(base_color_tex, material_sampler, o.a0).a * imp_inside(uv01.xy),
    textureSample(base_color_tex, material_sampler, o.a1).a * imp_inside(uv01.zw),
    textureSample(base_color_tex, material_sampler, o.a2).a * imp_inside(uv2),
  );
  o.wa = weights * c;
  o.alpha = (o.wa.x + o.wa.y + o.wa.z) * material.base_color.a;
  return o;
}

// The cut: the material's alpha test, half coverage without one.
fn imp_cutoff() -> f32 {
  return select(0.5, material.emissive.w, material.emissive.w >= 0.0);
}

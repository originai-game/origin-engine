// Whether this pipeline draws made materials (pipeline_features.rs).
override OG_SURFACE: bool = true;

// A made surface: what it does to the shading surface, the height that
// bends its normal (in units of the pattern's scale) and its wrapped light.
struct MadeSurface {
  s: Surface,
  height: f32,
  wrap: f32,
  tint: vec3<f32>,
};

// MaterialStyle.surface at strength `k` on the point `p` (the mesh's own
// position over the pattern's scale): 1 clay, 2 paper, 3 plastic. `detail`
// (0 to 1) is how much of the pattern this pixel resolves: the height and
// the colour grain go with it, the light response stays.
fn material_surface(kind: u32, k: f32, p: vec3<f32>, surface: Surface, detail: f32) -> MadeSurface {
  let kd = k * detail;
  var o: MadeSurface;
  o.s = surface;
  o.tint = vec3<f32>(1.0);
  if (kind == 1u) {
    // Clay: soft lumps over two octaves, thumb-pressed dents, matte, the
    // light wrapping warm past the terminator.
    let lump = value_noise3(p, 11u) * 0.65 + value_noise3(p * 2.3 + 7.1, 12u) * 0.35;
    let dent = smoothstep(0.6, 0.85, value_noise3(p * 1.6 + 3.7, 13u));
    o.height = (lump * 0.45 - dent * 0.3) * kd;
    o.s.roughness = max(o.s.roughness, 0.75);
    o.s.metallic = 0.0;
    o.s.albedo = o.s.albedo * (1.0 + (lump - 0.5) * 0.25 * kd - dent * 0.12 * kd);
    o.wrap = 0.5 * k;
    o.tint = vec3<f32>(1.0, 0.72, 0.55);
  } else if (kind == 2u) {
    // Paper: a mottled fibre grain with streaks along one direction, in
    // the colour and the normal, fully rough, a little light through.
    let mottle = value_noise3(p, 21u);
    let fibre = value_noise3(p * vec3<f32>(3.0, 0.5, 3.0) + 1.3, 22u);
    let g = mottle * 0.55 + fibre * 0.45;
    o.height = g * 0.2 * kd;
    o.s.roughness = 1.0;
    o.s.metallic = 0.0;
    o.s.albedo = o.s.albedo * (1.0 + (g - 0.5) * 0.6 * kd);
    o.wrap = 0.25 * k;
    o.tint = vec3<f32>(1.0, 0.96, 0.88);
  } else {
    // Toy plastic or painted wood: a satin sheen, a little more colour,
    // a faint moulding ripple, a soft wrap.
    o.height = value_noise3(p, 31u) * 0.04 * kd;
    o.s.roughness = mix(o.s.roughness, clamp(o.s.roughness, 0.3, 0.45), k);
    o.s.metallic = o.s.metallic * (1.0 - k);
    let grey = luma(o.s.albedo);
    o.s.albedo = max(mix(vec3<f32>(grey), o.s.albedo, 1.0 + 0.35 * k), vec3<f32>(0.0));
    o.wrap = 0.3 * k;
    o.tint = vec3<f32>(1.0, 0.9, 0.82);
  }
  return o;
}

// A generated surface has no StandardMaterial to carry a flat-normal bit.
// Keep its existing orientation (including the back of a two-sided blade).
fn style_face_normal(n: vec3<f32>, dx: vec3<f32>, dy: vec3<f32>) -> vec3<f32> {
  if (frame.toon.z <= 0.5) { return n; }
  let face = cross(dx, dy);
  if (dot(face, face) <= 1e-20) { return n; }
  return normalize(face) * select(-1.0, 1.0, dot(face, n) >= 0.0);
}

struct StyledSurface {
  s: Surface,
  n: vec3<f32>,
  wrap: f32,
  tint: vec3<f32>,
};

// The same made material on a generated or instanced surface. Its owner
// supplies anchored pattern coordinates and their derivatives; world-space
// derivatives turn the pattern's height into a shading-normal perturbation.
// Called after weathering, without changing geometry or its shadow normal.
fn styled_surface(s: Surface, n: vec3<f32>, p: vec3<f32>, dx: vec3<f32>, dy: vec3<f32>, local_dx: vec3<f32>, local_dy: vec3<f32>) -> StyledSurface {
  var o = StyledSurface(s, n, 0.0, vec3<f32>(1.0));
  let ms = frame.material_surface;
  if (OG_SURFACE && ms.x > 0.5 && ms.y > 0.0) {
    let footprint = max(length(local_dx), length(local_dy));
    let fade = clamp((ms.z / max(footprint, 1e-6) - 3.0) / 6.0, 0.0, 1.0);
    let made = material_surface(u32(ms.x + 0.5), ms.y, p / ms.z, s, fade);
    o.s = made.s;
    o.wrap = made.wrap;
    o.tint = made.tint;
    let height = made.height * ms.z;
    let r1 = cross(dy, n);
    let r2 = cross(n, dx);
    let det = dot(dx, r1);
    let grad = sign(det) * (dpdxFine(height) * r1 + dpdyFine(height) * r2);
    let facing = abs(det) / max(length(cross(dx, dy)), 1e-20);
    let bumped = abs(det) * n - grad * smoothstep(0.02, 0.1, facing);
    o.n = select(n, normalize(bumped), dot(bumped, bumped) > 1e-30);
    o.s.f0 = mix(vec3<f32>(0.04), o.s.albedo, o.s.metallic);
  }
  return o;
}

// The energy-conserving wrapped diffuse lobe already used by model
// materials: share it with generated surfaces without changing toon bands.
fn styled_wrap(made: StyledSurface, l: vec3<f32>, shadow: f32) -> vec3<f32> {
  let raw = dot(made.n, l);
  let wrapped = max(raw + made.wrap, 0.0) / ((1.0 + made.wrap) * (1.0 + made.wrap));
  let extra = wrapped - max(raw, 0.0);
  let gain = max(extra, 0.0) * made.tint + vec3<f32>(min(extra, 0.0));
  return made.s.albedo / PI * (1.0 - made.s.metallic) * gain * frame.light_color.rgb * select(1.0, shadow, raw > 0.0);
}

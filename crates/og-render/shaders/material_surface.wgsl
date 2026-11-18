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

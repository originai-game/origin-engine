// Joint palette, morph deltas and baked clips as data textures of vec4s
// (data_texture.rs, a matrix four columns in a row): a vertex stage reads
// no storage under WebGPU's compatibility mode.
@group(SKIN_GROUP) @binding(0) var joint_palette: texture_2d<f32>;
@group(SKIN_GROUP) @binding(1) var morph_deltas: texture_2d<f32>;
@group(SKIN_GROUP) @binding(2) var baked_palette: texture_2d<f32>;

fn joint_matrix(i: u32) -> mat4x4<f32> {
  let b = i * 4u;
  return mat4x4<f32>(
    textureLoad(joint_palette, data_at(b), 0),
    textureLoad(joint_palette, data_at(b + 1u), 0),
    textureLoad(joint_palette, data_at(b + 2u), 0),
    textureLoad(joint_palette, data_at(b + 3u), 0),
  );
}

fn baked_matrix(i: u32) -> mat4x4<f32> {
  let b = i * 4u;
  return mat4x4<f32>(
    textureLoad(baked_palette, data_at(b), 0),
    textureLoad(baked_palette, data_at(b + 1u), 0),
    textureLoad(baked_palette, data_at(b + 2u), 0),
    textureLoad(baked_palette, data_at(b + 3u), 0),
  );
}

// Joint `j` of a skinned instance's palette (skin.rs, the palette
// buffer's layout): palette.x its first slot, palette.y its joints. A
// baked clip (palette.w below -0.5) blends that frame's joint with the
// next frame's, palette.y slots on, at -palette.w - 1.
fn palette_joint(palette: vec4<f32>, j: u32) -> mat4x4<f32> {
  let base = u32(palette.x);
  let count = u32(max(palette.y, 1.0));
  let k = min(j, count - 1u);
  if (palette.w < -0.5) {
    let t = -palette.w - 1.0;
    return baked_matrix(base + k) * (1.0 - t) + baked_matrix(base + count + k) * t;
  }
  return joint_matrix(base + k);
}

// The weighted joint matrices of one vertex (identity without weights).
fn skin_matrix(joints: vec4<u32>, weights: vec4<f32>, palette: vec4<f32>) -> mat4x4<f32> {
  let total = weights.x + weights.y + weights.z + weights.w;
  if (total < 1e-6) {
    return mat4x4<f32>(vec4<f32>(1.0, 0.0, 0.0, 0.0), vec4<f32>(0.0, 1.0, 0.0, 0.0), vec4<f32>(0.0, 0.0, 1.0, 0.0), vec4<f32>(0.0, 0.0, 0.0, 1.0));
  }
  let w = weights / total;
  // A joint without weight adds nothing (x + 0·m is x, exactly): its two
  // baked frames are not read. A rigid part (a crowd member's, a prop's)
  // reads one joint, not four.
  var m = palette_joint(palette, joints.x) * w.x;
  if (w.y != 0.0) { m += palette_joint(palette, joints.y) * w.y; }
  if (w.z != 0.0) { m += palette_joint(palette, joints.z) * w.z; }
  if (w.w != 0.0) { m += palette_joint(palette, joints.w) * w.w; }
  return m;
}

fn morph_delta(i: u32) -> vec4<f32> {
  return textureLoad(morph_deltas, data_at(i), 0);
}

// Blendshape weight `t` of the weights from palette slot `slot` (16 a
// slot, column-major: skin.rs, the palette buffer's layout).
fn morph_weight(slot: u32, t: u32) -> f32 {
  let m = joint_matrix(slot + t / 16u);
  return m[(t % 16u) / 4u][t % 4u];
}

struct Morphed {
  position: vec3<f32>,
  normal: vec3<f32>,
};

// Blendshapes: vertex `vi`'s position and normal with its morph deltas
// added at their weights. palette.z: the mesh's first morph header,
// palette.w: its targets (0: none); `weights`: the first weight slot plus
// one (0: no weights). Each vertex walks only the targets that move it.
// `deformed`: the first slot plus one of the instance's own vertex
// positions (a simulated garment: skin.rs, the palette buffer's layout),
// which replace the mesh's and its morphs (0: none). Two vertices a slot:
// columns 0 and 1 the even vertex's position and normal, 2 and 3 the odd's.
fn morphed(position: vec3<f32>, normal: vec3<f32>, vi: u32, palette: vec4<f32>, weights: f32, deformed: f32) -> Morphed {
  var m: Morphed;
  if (deformed > 0.5) {
    let d = joint_matrix(u32(deformed) - 1u + vi / 2u);
    let c = (vi % 2u) * 2u;
    m.position = d[c].xyz;
    m.normal = d[c + 1u].xyz;
    return m;
  }
  m.position = position;
  m.normal = normal;
  if (palette.w < 0.5 || weights < 0.5) { return m; }
  let slot = u32(weights) - 1u;
  let h = morph_delta(u32(palette.z) + vi);
  let first = u32(h.x);
  let count = u32(h.y);
  for (var k = 0u; k < count; k++) {
    let d = morph_delta(first + k * 2u);
    let w = morph_weight(slot, u32(d.w));
    if (w != 0.0) {
      m.position += d.xyz * w;
      m.normal += morph_delta(first + k * 2u + 1u).xyz * w;
    }
  }
  return m;
}

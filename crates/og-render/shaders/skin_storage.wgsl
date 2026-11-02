// Joint palette and morph deltas as read-only storage buffers (WebGPU,
// Metal, Vulkan).
@group(SKIN_GROUP) @binding(0) var<storage, read> joint_palette: array<mat4x4<f32>>;
@group(SKIN_GROUP) @binding(1) var<storage, read> morph_deltas: array<vec4<f32>>;

fn joint_matrix(i: u32) -> mat4x4<f32> {
  return joint_palette[i];
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
    return joint_palette[base + k] * (1.0 - t) + joint_palette[base + count + k] * t;
  }
  return joint_palette[base + k];
}

fn morph_delta(i: u32) -> vec4<f32> {
  return morph_deltas[i];
}

// Blendshape weight `t` of the weights from palette slot `slot` (16 a
// slot, column-major: skin.rs, the palette buffer's layout).
fn morph_weight(slot: u32, t: u32) -> f32 {
  let m = joint_palette[slot + t / 16u];
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
    let d = joint_palette[u32(deformed) - 1u + vi / 2u];
    let c = (vi % 2u) * 2u;
    m.position = d[c].xyz;
    m.normal = d[c + 1u].xyz;
    return m;
  }
  m.position = position;
  m.normal = normal;
  if (palette.w < 0.5 || weights < 0.5) { return m; }
  let slot = u32(weights) - 1u;
  let h = morph_deltas[u32(palette.z) + vi];
  let first = u32(h.x);
  let count = u32(h.y);
  for (var k = 0u; k < count; k++) {
    let d = morph_deltas[first + k * 2u];
    let w = morph_weight(slot, u32(d.w));
    if (w != 0.0) {
      m.position += d.xyz * w;
      m.normal += morph_deltas[first + k * 2u + 1u].xyz * w;
    }
  }
  return m;
}

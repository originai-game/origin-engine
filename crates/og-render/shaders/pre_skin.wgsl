// Single-pass compute pre-skinning (`mesh/pre_skin.rs`): evaluates 4-bone
// linear blend skinning and blendshape morph deltas once per frame per dynamic
// character into the joint palette's output slots (current pose and, when TAA
// or motion blur requests it, previous world-space pose), eliminating 7-15x
// redundant vertex-stage skinning across cascades, local shadows, prepass,
// main, reflections, probes, and secondary views.
//
// Prepended by `common.wgsl` (`data_at`, `safe_normalize`, `oct_encode`).

struct PreSkinParams {
  // x: vertex_count, y: palette_first, z: palette_count, w: out_slot (current)
  head: vec4<u32>,
  // x: morph_first, y: morph_targets, z: weights_slot (0 = none), w: section (0 = no prev)
  morph: vec4<u32>,
  // Instance model matrix (used for previous world-space fallback of unweighted vertices).
  model0: vec4<f32>,
  model1: vec4<f32>,
  model2: vec4<f32>,
  model3: vec4<f32>,
};

@group(0) @binding(0) var<storage, read> joints_buf: array<vec4<f32>>;
@group(0) @binding(1) var<storage, read> vertices_buf: array<f32>;
@group(0) @binding(2) var<storage, read> skin_buf: array<u32>;
@group(0) @binding(3) var<uniform> params: PreSkinParams;
@group(0) @binding(4) var morph_deltas: texture_2d<f32>;
@group(0) @binding(5) var palette_out: texture_storage_2d<rgba32float, write>;

fn read_mat(slot: u32) -> mat4x4<f32> {
  let b = slot * 4u;
  return mat4x4<f32>(
    joints_buf[b],
    joints_buf[b + 1u],
    joints_buf[b + 2u],
    joints_buf[b + 3u],
  );
}

fn read_weight(slot: u32, t: u32) -> f32 {
  let b = (slot + t / 16u) * 4u + (t % 16u) / 4u;
  return joints_buf[b][t % 4u];
}

fn read_morph(i: u32) -> vec4<f32> {
  return textureLoad(morph_deltas, data_at(i), 0);
}

fn eval_skin(joints: vec4<u32>, weights: vec4<f32>, base: u32, count: u32) -> mat4x4<f32> {
  let total = weights.x + weights.y + weights.z + weights.w;
  if (total < 1e-6 || count == 0u) {
    return mat4x4<f32>(
      vec4<f32>(1.0, 0.0, 0.0, 0.0),
      vec4<f32>(0.0, 1.0, 0.0, 0.0),
      vec4<f32>(0.0, 0.0, 1.0, 0.0),
      vec4<f32>(0.0, 0.0, 0.0, 1.0),
    );
  }
  let w = weights / total;
  let last = count - 1u;
  var m = read_mat(base + min(joints.x, last)) * w.x;
  if (w.y != 0.0) { m += read_mat(base + min(joints.y, last)) * w.y; }
  if (w.z != 0.0) { m += read_mat(base + min(joints.z, last)) * w.z; }
  if (w.w != 0.0) { m += read_mat(base + min(joints.w, last)) * w.w; }
  return m;
}

fn cofactor3(m: mat3x3<f32>) -> mat3x3<f32> {
  return mat3x3<f32>(
    cross(m[1], m[2]),
    cross(m[2], m[0]),
    cross(m[0], m[1]),
  );
}

fn eval_morph(pos: vec3<f32>, norm: vec3<f32>, vi: u32, weights_slot: u32) -> array<vec3<f32>, 2> {
  var p = pos;
  var n = norm;
  if (params.morph.y > 0u && weights_slot > 0u) {
    let slot = weights_slot - 1u;
    let h = read_morph(params.morph.x + vi);
    let first = u32(h.x);
    let count = u32(h.y);
    for (var k = 0u; k < count; k++) {
      let d = read_morph(first + k * 2u);
      let w = read_weight(slot, u32(d.w));
      if (w != 0.0) {
        p += d.xyz * w;
        n += read_morph(first + k * 2u + 1u).xyz * w;
      }
    }
  }
  return array<vec3<f32>, 2>(p, n);
}

@compute @workgroup_size(64, 1, 1)
fn cs_pre_skin(@builtin(global_invocation_id) id: vec3<u32>) {
  let vi = id.x;
  if (vi >= params.head.x) { return; }

  // Unpack Vertex (16 f32s = 64 bytes).
  let vb = vi * 16u;
  let pos = vec3<f32>(vertices_buf[vb], vertices_buf[vb + 1u], vertices_buf[vb + 2u]);
  let norm = vec3<f32>(vertices_buf[vb + 3u], vertices_buf[vb + 4u], vertices_buf[vb + 5u]);
  let tan = vec4<f32>(
    vertices_buf[vb + 12u],
    vertices_buf[vb + 13u],
    vertices_buf[vb + 14u],
    vertices_buf[vb + 15u],
  );

  // Unpack SkinVertex (2 u32 packed u16x4 joints + 4 f32 weights = 6 u32s).
  let sb = vi * 6u;
  let j01 = skin_buf[sb];
  let j23 = skin_buf[sb + 1u];
  let joints = vec4<u32>(j01 & 0xffffu, j01 >> 16u, j23 & 0xffffu, j23 >> 16u);
  let weights = vec4<f32>(
    bitcast<f32>(skin_buf[sb + 2u]),
    bitcast<f32>(skin_buf[sb + 3u]),
    bitcast<f32>(skin_buf[sb + 4u]),
    bitcast<f32>(skin_buf[sb + 5u]),
  );

  // Current pose: morph + 4-bone LBS into palette_out[out_slot + vi / 2].
  let mn = eval_morph(pos, norm, vi, params.morph.z);
  let m = eval_skin(joints, weights, params.head.y, params.head.z);
  let m3 = mat3x3<f32>(m[0].xyz, m[1].xyz, m[2].xyz);
  let posed_pos = (m * vec4<f32>(mn[0], 1.0)).xyz;
  let posed_norm = safe_normalize(cofactor3(m3) * mn[1], mn[1]);
  let tan_len2 = dot(tan.xyz, tan.xyz);
  var posed_tan = vec3<f32>(1.0, 0.0, 0.0);
  var posed_tan_w = 0.0;
  if (tan_len2 > 1e-20 && tan.w != 0.0) {
    let pt = (m * vec4<f32>(tan.xyz, 0.0)).xyz;
    posed_tan = safe_normalize(pt, vec3<f32>(1.0, 0.0, 0.0));
    let hand = select(-1.0, 1.0, determinant(m3) >= 0.0);
    posed_tan_w = tan.w * hand;
  }

  let packed_tan = bitcast<f32>(pack2x16snorm(oct_encode(posed_tan)));
  let tex_base = (params.head.w + vi / 2u) * 4u + (vi % 2u) * 2u;
  textureStore(palette_out, vec2<i32>(data_at(tex_base)), vec4<f32>(posed_pos, packed_tan));
  textureStore(palette_out, vec2<i32>(data_at(tex_base + 1u)), vec4<f32>(posed_norm, posed_tan_w));

  // Previous world-space pose for TAA / motion vectors when `section > 0`.
  let section = params.morph.w;
  if (section > 0u) {
    let prev_w_slot = select(0u, params.morph.z + section, params.morph.z > 0u);
    let prev_mn = eval_morph(pos, norm, vi, prev_w_slot);
    let weighted = (weights.x + weights.y + weights.z + weights.w) >= 1e-6;
    var prev_world: vec3<f32>;
    if (weighted) {
      let pm = eval_skin(joints, weights, params.head.y + section, params.head.z);
      prev_world = (pm * vec4<f32>(prev_mn[0], 1.0)).xyz;
    } else {
      let inst = mat4x4<f32>(params.model0, params.model1, params.model2, params.model3);
      prev_world = (inst * vec4<f32>(prev_mn[0], 1.0)).xyz;
    }
    let prev_base = (params.head.w + section + vi / 2u) * 4u + (vi % 2u) * 2u;
    textureStore(palette_out, vec2<i32>(data_at(prev_base)), vec4<f32>(prev_world, 0.0));
  }
}

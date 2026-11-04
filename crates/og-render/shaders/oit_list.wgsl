// The linked-list mode's exact resolve (oit_resolve.wgsl `oit_exact`):
// a pixel's nodes (oit_arena.wgsl's layout, read-only here as
// `oit_nodes`) in front of the opaque depth at the sample, sorted front
// to back by depth with a stable key for equal depths, composited in
// order. Adapted from Bevy's OIT resolve (MIT/Apache-2.0) and WebGPU
// Samples' A-buffer (BSD-3-Clause); notices in third_party/oit/. Both
// drop fragments once a global pool is full; here a pixel past its own
// nodes is resolved whole from the weighted sums instead.

// The most nodes a pixel sorts; oit.rs caps the nodes per pixel at it.
const OIT_SORT_LIMIT: u32 = 32u;
const OIT_HEADER_WORDS: u32 = 4u;
const OIT_NODE_WORDS: u32 = 7u;

struct OitNode {
  depth: f32,
  color: vec4<f32>,
  key: vec2<u32>,
};

// Front to back. Equal depths go by the fragments' keys (oit.wgsl
// `oit_key`: the draw's transform, then the primitive, with the sample
// mask in key.y's high bits), never by the order they arrived in nor by
// their colours. Only one primitive drawn twice under one transform keeps
// equal keys, and the insertion sort keeps those as they came.
fn oit_before(a: OitNode, b: OitNode) -> bool {
  if (a.depth != b.depth) { return a.depth < b.depth; }
  if (a.key.x != b.key.x) { return a.key.x < b.key.x; }
  return a.key.y < b.key.y;
}

fn oit_exact(p: vec2<i32>, sample: u32) -> vec4<f32> {
  let width = oit_nodes[0];
  let height = oit_nodes[1];
  let budget = oit_nodes[2];
  let pixel = u32(p.y) * width + u32(p.x);
  let count = oit_nodes[OIT_HEADER_WORDS + pixel];
  if (count > min(budget, OIT_SORT_LIMIT)) { return vec4<f32>(-1.0); }
  // Fragment stores need not wait for the depth test, so a node may lie
  // behind the opaque scene: tested here against each sample's depth
  // (the pass's depth test is Less), and kept only where it covered it.
  let opaque = oit_depth_at(p, sample);
  let base = OIT_HEADER_WORDS + width * height + pixel * budget * OIT_NODE_WORDS;
  var sorted: array<OitNode, OIT_SORT_LIMIT>;
  var used = 0u;
  for (var i = 0u; i < count; i++) {
    let at = base + i * OIT_NODE_WORDS;
    let depth = bitcast<f32>(oit_nodes[at]);
    let key = vec2<u32>(oit_nodes[at + 5u], oit_nodes[at + 6u]);
    if (!(depth < opaque) || ((key.y >> 28u) & (1u << sample)) == 0u) { continue; }
    let node = OitNode(depth, vec4<f32>(
      bitcast<f32>(oit_nodes[at + 1u]),
      bitcast<f32>(oit_nodes[at + 2u]),
      bitcast<f32>(oit_nodes[at + 3u]),
      bitcast<f32>(oit_nodes[at + 4u])), key);
    var j = used;
    while (j > 0u) {
      if (!oit_before(node, sorted[j - 1u])) { break; }
      sorted[j] = sorted[j - 1u];
      j -= 1u;
    }
    sorted[j] = node;
    used += 1u;
  }
  var result = vec4<f32>(0.0);
  for (var i = 0u; i < used; i++) {
    let c = sorted[i].color;
    let share = c.a * (1.0 - result.a);
    result += vec4<f32>(c.rgb * share, share);
  }
  return result;
}

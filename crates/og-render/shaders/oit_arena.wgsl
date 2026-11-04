// The linked-list mode's per-pixel fragment nodes (oit.rs `Arena`), in
// the participants' modules after oit.wgsl. Only their `*_oit_list`
// entry points read it, so the opaque and weighted pipelines keep the
// frame group without binding 12. The pixel-local storage follows Bevy's
// OIT (MIT/Apache-2.0) and WebGPU Samples' A-buffer (BSD-3-Clause), with
// the engine's bounded allocation; notices in third_party/oit/.
//
// One arena of words: a header (the pass's width, height and nodes per
// pixel, then the count of pixels that overflowed), one fragment count per
// pixel, then each pixel's own nodes, seven words each: the window depth,
// the straight RGBA colour (f32), and the fragment's key (oit.wgsl
// `oit_key`) with its raster sample mask in the high four bits of the
// second word. A pixel's nodes are its own, so the order the GPU runs
// fragments in never decides which pixel keeps a fragment: a pixel keeps
// its first nodes-per-pixel fragments, and one with more is counted once
// in the header and resolved whole from the weighted targets.
@group(0) @binding(12) var<storage, read_write> oit_arena: array<atomic<u32>>;

const OIT_HEADER_WORDS: u32 = 4u;
const OIT_NODE_WORDS: u32 = 7u;

fn oit_store(position: vec4<f32>, color: vec4<f32>, mask: u32, key: vec2<u32>) {
  let alpha = clamp(color.a, 0.0, 1.0);
  // What the weighted targets add nothing for is no fragment here either.
  if (!(alpha > 0.0) || mask == 0u) { return; }
  let width = atomicLoad(&oit_arena[0]);
  let height = atomicLoad(&oit_arena[1]);
  let budget = atomicLoad(&oit_arena[2]);
  let px = vec2<u32>(position.xy);
  if (px.x >= width || px.y >= height) { return; }
  let p = px.y * width + px.x;
  let slot = atomicAdd(&oit_arena[OIT_HEADER_WORDS + p], 1u);
  if (slot >= budget) {
    if (slot == budget) { atomicAdd(&oit_arena[3], 1u); }
    return;
  }
  let node = OIT_HEADER_WORDS + width * height + (p * budget + slot) * OIT_NODE_WORDS;
  let rgb = clamp(color.rgb, vec3<f32>(0.0), vec3<f32>(65504.0));
  atomicStore(&oit_arena[node], bitcast<u32>(position.z));
  atomicStore(&oit_arena[node + 1u], bitcast<u32>(rgb.r));
  atomicStore(&oit_arena[node + 2u], bitcast<u32>(rgb.g));
  atomicStore(&oit_arena[node + 3u], bitcast<u32>(rgb.b));
  atomicStore(&oit_arena[node + 4u], bitcast<u32>(alpha));
  atomicStore(&oit_arena[node + 5u], key.x);
  atomicStore(&oit_arena[node + 6u], (key.y & 0x0fffffffu) | ((mask & 15u) << 28u));
}

// Hybrid transparency (Maule, Comba, Torchelsen and Bastos, I3D 2013),
// with one exact layer: the weighted mode's front layer. Where the device
// writes storage from fragment shaders, the arena of the weighted mode is
// the header and one word a pixel (oit.rs `ArenaSize::front`). The front
// pass (`*_oit_mark`, no colour targets) leaves in each pixel's word the
// nearest depth of a fragment at least OIT_FRONT_ALPHA opaque, inverted so
// a cleared word (0) is none and atomicMax keeps the nearest: positive
// floats order as their bits, and the maximum does not depend on the
// order the fragments came in. The OIT pass (`*_oit_front`) sends the
// fragment at that depth to the front target and every other one to the
// weighted tail; the resolve composites the front over the tail, so the
// nearest surface (a billboard before glass) keeps its own colour rather
// than the weighted mean of everything in the pixel.

// The least alpha that makes a fragment the front layer: a faint haze in
// front (smoke) does not take the place of a surface behind it, and is
// composited in the tail at an error of at most its own alpha.
const OIT_FRONT_ALPHA: f32 = 0.125;
// How far (in units of the depth's last place) a fragment may lie behind
// the marked depth and still be the front: the two passes compute one
// fragment's depth in two pipelines. Fragments that close are coplanar
// and share the front, averaged (`oit_front_of` in both, order-free).
const OIT_FRONT_ULPS: u32 = 4u;

fn oit_front_word(position: vec4<f32>) -> u32 {
  let width = atomicLoad(&oit_arena[0]);
  let height = atomicLoad(&oit_arena[1]);
  let px = vec2<u32>(position.xy);
  if (px.x >= width || px.y >= height) { return 0xffffffffu; }
  return OIT_HEADER_WORDS + px.y * width + px.x;
}

fn oit_front_mark(position: vec4<f32>, alpha: f32) {
  if (!(clamp(alpha, 0.0, 1.0) >= OIT_FRONT_ALPHA)) { return; }
  let w = oit_front_word(position);
  if (w == 0xffffffffu) { return; }
  atomicMax(&oit_arena[w], ~bitcast<u32>(max(position.z, 0.0)));
}

fn oit_front_of(position: vec4<f32>, alpha: f32) -> bool {
  if (!(clamp(alpha, 0.0, 1.0) >= OIT_FRONT_ALPHA)) { return false; }
  let w = oit_front_word(position);
  if (w == 0xffffffffu) { return false; }
  let marked = atomicLoad(&oit_arena[w]);
  if (marked == 0u) { return false; }
  return bitcast<u32>(max(position.z, 0.0)) <= ~marked + OIT_FRONT_ULPS;
}

// A fragment of the weighted mode with the front layer.
fn oit_hybrid(color: vec4<f32>, position: vec4<f32>) -> OitOutput {
  if (oit_front_of(position, color.a)) {
    return oit_front(color);
  }
  return oit_weighted(color, position.z);
}

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

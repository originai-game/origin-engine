// Order-independent transparency, the fragment side every participant
// shares (oit.rs): the lit modules, the sprite, particle and ribbon
// modules. Weighted blending after McGuire and Bavoil, "Weighted Blended
// Order-Independent Transparency", JCGT 2(2), 2013, with the depth weight
// of McGuire's 2015 note; written for the engine (third_party/oit/).
//
// Every transparent fragment of a frame is accumulated here, in both
// transparency modes: into `accum` (RGBA16F, added) its premultiplied
// colour and alpha times its weight, into `reveal` (R16F, multiplied) the
// share of what lies behind that it lets through. The linked-list mode
// also keeps the fragment in its pixel's nodes (oit_arena.wgsl) and uses
// these only where the pixel ran out of nodes (oit_resolve.wgsl).

// Set by a lit module's OIT entry points before shading: the screen's
// AO and contact shadows are the opaque surface's behind the fragment,
// not its own (pbr_fs.wgsl, physical.wgsl, lib.rs `screen_ao`).
var<private> oit_pass: bool;

// What a fragment adds to the OIT pass's targets: `accum` the weighted
// tail's colour and alpha times weight, `reveal` the transmittances the
// tail (r) and the front layer (g) multiply in, `front` the front layer's
// premultiplied colour and alpha (oit_arena.wgsl `oit_hybrid`). A fragment
// adds to the tail or to the front, never both.
struct OitOutput {
  @location(0) accum: vec4<f32>,
  @location(1) reveal: vec2<f32>,
  @location(2) front: vec4<f32>,
};

// The weight of a fragment of `alpha` at window depth `depth`
// (McGuire's general-purpose weight over gl_FragCoord.z), scaled by
// 1 / 3e3 so the largest is 1: the scale cancels in the resolve and
// leaves the half-float sums headroom for bright, many-layered pixels.
fn oit_weight(alpha: f32, depth: f32) -> f32 {
  let a = min(1.0, alpha * 10.0) + 0.01;
  let z = 1.0 - clamp(depth, 0.0, 1.0) * 0.9;
  return clamp(a * a * a * 1e8 * z * z * z, 1e-2, 3e3) / 3e3;
}

// A fragment's straight colour `color` (as the sorted blending drew it,
// src alpha over the scene) into the weighted tail. The premultiplied
// colour is kept below 64 so a pixel of many bright layers stays finite.
fn oit_weighted(color: vec4<f32>, depth: f32) -> OitOutput {
  let alpha = clamp(color.a, 0.0, 1.0);
  let premultiplied = clamp(color.rgb * alpha, vec3<f32>(0.0), vec3<f32>(64.0));
  return OitOutput(
    vec4<f32>(premultiplied, alpha) * oit_weight(alpha, depth),
    vec2<f32>(alpha, 0.0),
    vec4<f32>(0.0));
}

// The same fragment as the pixel's front layer: composited exactly over
// the weighted tail in the resolve (oit_resolve.wgsl).
fn oit_front(color: vec4<f32>) -> OitOutput {
  let alpha = clamp(color.a, 0.0, 1.0);
  let premultiplied = clamp(color.rgb * alpha, vec3<f32>(0.0), vec3<f32>(64.0));
  return OitOutput(vec4<f32>(0.0), vec2<f32>(0.0, alpha), vec4<f32>(premultiplied, alpha));
}

// FNV-1a over a word.
fn oit_hash(h: u32, word: u32) -> u32 {
  return (h ^ word) * 16777619u;
}

// The stable identity of a fragment's source, for the linked-list
// resolve's equal-depth tie-break (oit_resolve.wgsl `oit_before`): x the
// draw's instance (its transform), y the primitive (its provoking vertex,
// in the draw's own space). Neither depends on the order the draws were
// submitted in. Vertex stages set it flat.
fn oit_key(model: mat4x4<f32>, local: vec3<f32>) -> vec2<u32> {
  var draw = 2166136261u;
  for (var c = 0u; c < 4u; c++) {
    for (var r = 0u; r < 3u; r++) {
      draw = oit_hash(draw, bitcast<u32>(model[c][r]));
    }
  }
  var primitive = 2166136261u;
  for (var r = 0u; r < 3u; r++) {
    primitive = oit_hash(primitive, bitcast<u32>(local[r]));
  }
  return vec2<u32>(draw, primitive);
}

// The same identity from four words of a draw (a sprite's or a particle's
// own record) and its primitive's position.
fn oit_key4(draw: vec4<f32>, local: vec3<f32>) -> vec2<u32> {
  var d = 2166136261u;
  for (var c = 0u; c < 4u; c++) {
    d = oit_hash(d, bitcast<u32>(draw[c]));
  }
  var primitive = 2166136261u;
  for (var r = 0u; r < 3u; r++) {
    primitive = oit_hash(primitive, bitcast<u32>(local[r]));
  }
  return vec2<u32>(d, primitive);
}

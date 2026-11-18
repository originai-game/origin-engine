// The grounded ground's openness (ground_heights.rs): for each texel of the
// ground's height map, the share of the sky's cosine-weighted light that
// what stands round it leaves open to the ground there. Baked when the map
// is drawn again; the sky pass reads it (sky_ground.wgsl `ground_open`).
//
// The horizon of a heightfield along a few directions (Max, "Horizon
// mapping", 1988; Timonen and Westerholm, "Scalable Height Field
// Self-Shadowing", 2010), each closing the cosine-weighted share above it
// as GTAO's slices do for a level receiver (Jimenez et al., "Practical
// Realtime Strategies for Accurate Indirect Occlusion", 2016): a horizon at
// elevation e leaves cos²(e) = 1 / (1 + tan²(e)) of a slice open. Each
// texel turns its directions by one of sixteen phases (a 4x4 pattern), so
// every 4x4 window holds sixteen times as many, which the sky pass averages.
// Prepended: GROUND_BELOW, GROUND_ABOVE (metres).

@group(0) @binding(0) var heights: texture_2d<f32>;
// x: the map's half side (metres).
@group(0) @binding(1) var<uniform> ground: vec4<f32>;

const GROUND_DIRS: u32 = 12u;
const GROUND_STEPS: u32 = 16u;
// How far round a point what stands on the ground is looked for (metres).
const GROUND_REACH: f32 = 12.0;

struct OpenOut {
  @builtin(position) pos: vec4<f32>,
};

@vertex
fn vs_ground_open(@builtin(vertex_index) i: u32) -> OpenOut {
  var o: OpenOut;
  let uv = vec2<f32>(f32((i << 1u) & 2u), f32(i & 2u));
  o.pos = vec4<f32>(uv * 2.0 - 1.0, 0.0, 1.0);
  return o;
}

// The height over the ground at texel `q` (metres): the map's depth read
// back (ground_heights.rs `ground_view`); a cleared texel is under it.
fn ground_height(q: vec2<i32>) -> f32 {
  let z = textureLoad(heights, q, 0).r;
  return (1.0 - z) * (GROUND_BELOW + GROUND_ABOVE) - GROUND_BELOW;
}

@fragment
fn fs_ground_open(in: OpenOut) -> @location(0) vec4<f32> {
  let size = vec2<i32>(textureDimensions(heights));
  let px = vec2<i32>(in.pos.xy);
  let texel = 2.0 * ground.x / f32(size.x);
  // The 4x4 pattern's phase (a Bayer matrix: neighbours far apart).
  let b = vec2<u32>(px) & vec2<u32>(3u);
  let bayer = array<f32, 16>(0.0, 8.0, 2.0, 10.0, 12.0, 4.0, 14.0, 6.0, 3.0, 11.0, 1.0, 9.0, 15.0, 7.0, 13.0, 5.0);
  let phase = (bayer[b.y * 4u + b.x] + 0.5) / 16.0;
  var open = 0.0;
  for (var d = 0u; d < GROUND_DIRS; d++) {
    let a = (f32(d) + phase) * 6.2831853 / f32(GROUND_DIRS);
    let along = vec2<f32>(cos(a), sin(a));
    var rise = 0.0;
    for (var s = 0u; s < GROUND_STEPS; s++) {
      let k = (f32(s) + 0.5) / f32(GROUND_STEPS);
      let dist = max(GROUND_REACH * k * k, texel);
      let q = vec2<i32>(floor(in.pos.xy + along * (dist / texel)));
      if (any(q < vec2<i32>(0)) || any(q >= size)) { break; }
      rise = max(rise, ground_height(q) / dist);
    }
    open += 1.0 / (1.0 + rise * rise);
  }
  return vec4<f32>(open / f32(GROUND_DIRS), 0.0, 0.0, 1.0);
}

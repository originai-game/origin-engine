// The clouds' shadow (clouds/shadow.rs), over one period of the cloud
// field (four tiles of the scale, which every scale the shape reads
// repeats in), so both maps tile over the world and the wind only shifts
// where they are read. A texel is where a ray to the sun leaves the
// layer's base, in the field's own x and z; the ray is marched from there
// to the layer's top through the cloud the sky draws (clouds_shape.wgsl
// `cloud_edge_in`) without its detail, which a texel (70 m at the default
// scale) is wider than. Two maps of the one march:
//   - shadow_out, how much of the sun's beam the layer lets through to the
//     world under it (lighting.wgsl `cloud_shadow`);
//   - light_out, the clouds' own shadow on themselves, a Beer shadow map
//     (Hillaire, "Physically Based and Scalable Atmospheres in Unreal
//     Engine", SIGGRAPH 2020; third_party/nubis-clouds.md): the cloud
//     along the ray as its moments over the layer's height, r its optical
//     depth, g and b that depth's first and second moments about the
//     layer's middle; a the optical depth of the cloud's envelope, the
//     cloud without its billows, which the light scattered many times
//     crosses (sky.wgsl `cloud_sunlit`: it has spread over hundreds of
//     metres, and does not draw each lump of a cloud's top on its base).
//     The march reads them between
//     texels and makes of them one slab of even density, the one with the
//     ray's density's mean height and spread (sky.wgsl `cloud_light_depth`):
//     moments blend as the cloud does, so the shadow of a cloud's edge is
//     half the cloud's (a slab's own top and bottom, blended, drew a step
//     in the cloud's light at every texel the ray crossed), and a wisp
//     above a cloud does not start its shadow there.
// Baked again only when the sun moves a step, never every frame.
// After frame.wgsl and clouds_shape.wgsl.

struct CloudShadowBake {
  // x, z: the ray's run per metre it rises; y: the wind's offset in
  // height; w: the steps. Each is rounded (shadow.rs `Bake::of`): the maps
  // are baked again when one moves a step, never every frame.
  ray: vec4<f32>,
};
@group(1) @binding(0) var<uniform> bake: CloudShadowBake;
@group(1) @binding(3) var shadow_out: texture_storage_2d<r32float, write>;
@group(1) @binding(4) var light_out: texture_storage_2d<rgba16float, write>;

const CLOUD_SHADOW_STEPS: u32 = 64u;

// The integral up to `x` of the density over the cloud's edge, as a share
// of the body's: nothing outside the cloud, rising evenly over the edge
// (0 to 1), whole past it.
fn edge_area(x: f32) -> f32 {
  let t = clamp(x, 0.0, 1.0);
  return 0.5 * t * t + max(x - 1.0, 0.0);
}

// The mean over a step of the density, as a share of the body's, of a
// cloud `below` past its surface at the step's start and `above` at its
// end (cloud_edge_in), taken to run evenly between.
fn edge_cover(below: f32, above: f32) -> f32 {
  if (abs(above - below) > 1e-4) { return (edge_area(above) - edge_area(below)) / (above - below); }
  return clamp(below, 0.0, 1.0);
}

@compute @workgroup_size(8, 8)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
  let side = textureDimensions(shadow_out);
  if (any(id.xy >= side)) { return; }
  let c = frame.clouds;
  let texel = 4.0 * c[1].w / f32(side.x);
  let base = (vec2<f32>(id.xy) + 0.5) * texel;
  let steps = min(u32(bake.ray.w), CLOUD_SHADOW_STEPS);
  // A step's length along the slanted ray.
  let ds = c[0].w / f32(steps) * sqrt(1.0 + dot(bake.ray.xz, bake.ray.xz));
  var depth = 0.0;
  var first = 0.0;
  var second = 0.0;
  var envelope = 0.0;
  // A step's cloud is the mean over it of the density, rising evenly over
  // the cloud's edge, of the edge's depth taken to run evenly between the
  // step's two ends: the depth of a cloud whose surface lies between two
  // samples follows where it lies. Summed from samples of the density, a
  // cloud's depth went up a whole step's at a time as its surface passed
  // each sample's height, and its light showed the steps as contour lines
  // across its base.
  var below = cloud_edge_in(vec3<f32>(base.x, c[0].z - bake.ray.y, base.y), 0.0, texel);
  for (var i = 0u; i < steps; i++) {
    let hf = (f32(i) + 0.5) / f32(steps);
    let rise = (f32(i) + 1.0) / f32(steps) * c[0].w;
    let at = vec3<f32>(base.x + bake.ray.x * rise, c[0].z + rise - bake.ray.y, base.y + bake.ray.z * rise);
    let above = cloud_edge_in(at, (f32(i) + 1.0) / f32(steps), texel);
    let thin = 0.5 * (below.z + above.z);
    let d = c[0].y * thin * edge_cover(below.x, above.x) * ds;
    envelope += c[0].y * thin * edge_cover(below.y, above.y) * ds;
    below = above;
    depth += d;
    first += d * (hf - 0.5);
    second += d * (hf - 0.5) * (hf - 0.5);
  }
  // (Held to what a half float keeps.)
  let kept = min(1.0, 2000.0 / max(depth, 1e-6));
  envelope = min(envelope, 2000.0);
  let through = exp(-depth);
  textureStore(shadow_out, vec2<i32>(id.xy), vec4<f32>(through, 0.0, 0.0, 1.0));
  textureStore(light_out, vec2<i32>(id.xy), vec4<f32>(depth * kept, first * kept, second * kept, envelope));
}

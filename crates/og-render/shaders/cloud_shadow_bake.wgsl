// The clouds' shadow (clouds/shadow.rs): how much of the sun's beam the
// layer lets through, over one period of the cloud field (four tiles of
// the noise, which every scale the shape reads repeats in), so the map
// tiles over the world and the wind only shifts where it is read
// (lighting.wgsl `cloud_shadow`). A texel is where a ray to the sun leaves
// the layer's base, in the noise's own x and z; the ray is marched from
// there to the layer's top through the density the sky draws
// (clouds_shape.wgsl), without the fine erosion a texel is too wide for.
// After frame.wgsl and clouds_shape.wgsl.

struct CloudShadowBake {
  // x, z: the ray's run per metre it rises; y: the wind's offset in
  // height; w: the steps. Each is rounded (shadow.rs `Bake::of`): the map
  // is baked again when one moves a step, never every frame.
  ray: vec4<f32>,
};
@group(1) @binding(0) var<uniform> bake: CloudShadowBake;
@group(1) @binding(3) var shadow_out: texture_storage_2d<r32float, write>;

const CLOUD_SHADOW_STEPS: u32 = 64u;

@compute @workgroup_size(8, 8)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
  let side = textureDimensions(shadow_out);
  if (any(id.xy >= side)) { return; }
  let c = frame.clouds;
  let base = (vec2<f32>(id.xy) + 0.5) / vec2<f32>(side) * 4.0 * c[1].w;
  let steps = min(u32(bake.ray.w), CLOUD_SHADOW_STEPS);
  // A step's length along the slanted ray.
  let ds = c[0].w / f32(steps) * sqrt(1.0 + dot(bake.ray.xz, bake.ray.xz));
  var depth = 0.0;
  for (var i = 0u; i < steps; i++) {
    let hf = (f32(i) + 0.5) / f32(steps);
    let rise = hf * c[0].w;
    let at = vec3<f32>(base.x + bake.ray.x * rise, c[0].z + rise - bake.ray.y, base.y + bake.ray.z * rise);
    depth += cloud_density_in(at, hf, true) * ds;
  }
  textureStore(shadow_out, vec2<i32>(id.xy), vec4<f32>(exp(-depth), 0.0, 0.0, 1.0));
}

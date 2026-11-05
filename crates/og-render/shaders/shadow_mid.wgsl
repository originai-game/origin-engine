// A local shadow face's midpoint (shadow_mid.rs): what its layer holds for
// the lit shaders, from its two rasters. The layer itself was drawn as it
// is, keeping the casters' far sides (the nearest surface facing away from
// the light); `near` holds the same face drawn mirrored in x, keeping their
// near sides (the nearest facing it). The layer becomes the depth halfway
// between them, Woo's midpoint shadow map ("The Shadow Depth Map Revisited",
// Graphics Gems III, 1992): inside a closed caster, at the plane of a
// zero-thickness one (both sides drawn: a double-sided card, or two
// coplanar quads facing apart), so no surface compares against itself.
// Where no far side was drawn (1: past the range) nothing is: a single-
// sided open surface (a plane, a wall) casts nothing, as it did when the
// layers held far sides alone, rather than shadowing the receivers it
// meets at a crease through the filter's taps on it. Where no near side
// was drawn, the far side is the depth.

@group(0) @binding(0) var layer: texture_storage_2d<r32float, read_write>;
@group(0) @binding(1) var near: texture_2d<f32>;

@compute @workgroup_size(8, 8)
fn resolve(@builtin(global_invocation_id) id: vec3<u32>) {
  let size = textureDimensions(layer);
  if (any(id.xy >= size)) { return; }
  let far = textureLoad(layer, id.xy).r;
  let front = textureLoad(near, vec2<u32>(size.x - 1u - id.x, id.y), 0).r;
  var mid = 0.5 * (far + front);
  if (front >= 1.0 || far >= 1.0) { mid = far; }
  textureStore(layer, id.xy, vec4<f32>(mid, 0.0, 0.0, 1.0));
}

// For material shaders (fragment hooks only): the frame's planar reflection
// (the water drawn with one) seen at world point `p` (where the mirrored
// camera sees `p`, as three.js Water's textureMatrix projects a point; for a
// point on the plane, its mirror image), shifted by `shift` in the texture's
// uv. rgb: the mirrored scene; a: 1 when there is one, 0 when the frame has
// none, -1 while the reflection itself is drawn (a reader of the mirror may
// discard itself there, as three.js's readers keep out of the mirror they
// read). Prepended to the lit shader (pbr.wgsl's `fx`) where the effect
// group has the slot (lib.rs `planar_slot`); elsewhere `PLANAR_STUB`.
@group(2) @binding(4) var planar_tex: texture_2d<f32>;

fn planar_reflection(p: vec3<f32>, shift: vec2<f32>) -> vec4<f32> {
  let n = fx.planar.xyz;
  if (dot(n, n) < 0.25) {
    return vec4<f32>(0.0, 0.0, 0.0, select(0.0, -1.0, fx.planar.w < -0.5));
  }
  let m = p - n * (2.0 * (dot(p, n) - fx.planar.w));
  let c = fx.view_proj_unjittered * vec4<f32>(m, 1.0);
  let ndc = c.xy / max(c.w, 1e-4);
  // The mirrored camera's image is flipped left to right (water.wgsl).
  let uv = vec2<f32>(0.5 - 0.5 * ndc.x, 0.5 - 0.5 * ndc.y) + shift;
  let rgb = textureSampleLevel(planar_tex, fx_sampler, clamp(uv, vec2<f32>(0.0), vec2<f32>(1.0)), 0.0).rgb;
  return vec4<f32>(rgb, 1.0);
}

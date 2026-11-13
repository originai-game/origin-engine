// The raymarch of volumetric.wgsl (VolumetricMethod::Raymarch): each step lit by the sun
// through its cascades and by the first RAYMARCH_LIGHTS (volumetric.rs)
// point, spot and rect lights of its cluster through their shadows and
// cookies.

@fragment
fn fs_raymarch(in: VolOut) -> @location(0) vec4<f32> {
  let d = min(pixel_depth(in.pos.xy), vol.extent.x);
  let uv = in.pos.xy / vol.extent.zw;
  let dir = view_ray(uv);
  let length = d / max(dot(dir, frame.view_forward.xyz), 1e-3);
  let steps = max(u32(vol.extent.y), 1u);
  let dt = length / f32(steps);
  // Interleaved gradient noise, turned each frame: the steps' banding
  // becomes grain TAA resolves.
  let jitter = fract(52.9829189 * fract(dot(in.pos.xy + vol.grid.w * 5.588238, vec2<f32>(0.06711056, 0.00583715))));
  var acc = vec3<f32>(0.0);
  var transmittance = 1.0;
  for (var i = 0u; i < steps; i++) {
    let p = frame.view_pos.xyz + dir * (f32(i) + jitter) * dt;
    let sigma = vol_density(p);
    if (sigma <= 0.0) { continue; }
    let tr = exp(-sigma * dt);
    // A step holds the water's light as far as its length.
    let s = vol.scatter.rgb * sigma * medium_in(p, dir, in.pos.xy, RAYMARCH_LIGHTS, 0.5 * dt);
    acc += transmittance * s * (1.0 - tr) / sigma;
    transmittance *= tr;
  }
  // The water's fog has dimmed the scene already.
  return vec4<f32>(acc, select(transmittance, 1.0, vol.water.x > 0.5));
}

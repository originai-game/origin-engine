// The raymarch of volumetric.wgsl (VolumetricMethod::Raymarch): each step lit by the sun
// through its cascades and by the first RAYMARCH_LIGHTS (volumetric.rs)
// point, spot and rect lights of its cluster through their shadows and
// cookies.

@fragment
fn fs_raymarch(in: VolOut) -> @location(0) vec4<f32> {
  let depth = pixel_depth(in.pos.xy);
  let d = min(depth, vol.extent.x);
  let uv = in.pos.xy / vol.extent.zw;
  let dir = view_ray(uv);
  let env = env_in(frame.view_pos.xyz);
  let length = d / max(dot(dir, frame.view_forward.xyz), 1e-3);
  let steps = max(u32(vol.extent.y), 1u);
  let dt = length / f32(steps);
  // Interleaved gradient noise, turned each frame: the steps' banding
  // becomes grain TAA resolves.
  let jitter = ign(in.pos.xy + vol.grid.w * 5.588238);
  var acc = vec3<f32>(0.0);
  var transmittance = 1.0;
  for (var i = 0u; i < steps; i++) {
    let p = frame.view_pos.xyz + dir * (f32(i) + jitter) * dt;
    let m = vol_medium(p);
    let sigma = m.sigma;
    if (sigma <= 0.0) { continue; }
    let tr = exp(-sigma * dt);
    // A step holds the water's light as far as its length.
    var s = m.own + m.scatter * medium_in(p, dir, in.pos.xy, RAYMARCH_LIGHTS, 0.5 * dt);
    if (m.local.x + m.local.y + m.local.z > 0.0) { s += m.local * ambient_in(p, env); }
    acc += transmittance * s * (1.0 - tr) / sigma;
    transmittance *= tr;
  }
  // The air past the march, behind what it holds.
  let far = far_air(dir, depth);
  acc += transmittance * far.rgb;
  transmittance *= far.a;
  // The water's fog has dimmed the scene already.
  return vec4<f32>(acc, select(transmittance, 1.0, VOL_WATER));
}

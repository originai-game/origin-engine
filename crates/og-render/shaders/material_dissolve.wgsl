// After the material's textures (material_maps.wgsl or
// material_physical.wgsl), which define dissolve_noise.

// The dissolve at a fragment: rgb the glowing edge's emission, w 1 when
// the fragment is dissolved. Callers fold w into their entry point's one
// `discard`: on Metal a second discard in an entry point did not take
// effect, and dissolved pixels drew black.
fn dissolve_at(uv: vec2<f32>, local_pos: vec3<f32>) -> vec4<f32> {
  let amount = material.rim.w;
  if (amount <= 0.0) { return vec4<f32>(0.0); }
  let noise = dissolve_noise(uv, local_pos);
  if (noise < amount) { return vec4<f32>(0.0, 0.0, 0.0, 1.0); }
  let width = max(material.flash.w, 1e-4);
  let edge = 1.0 - clamp((noise - amount) / width, 0.0, 1.0);
  return vec4<f32>(material.dissolve.rgb * edge * edge, 0.0);
}


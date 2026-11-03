// Grass in the first shadow cascade (grass.rs, a field that casts):
// depth only, each blade built as the sun sees it (turned towards it when
// edge-on, at no narrowest width). Prepended by shadow_pass.wgsl (group
// 0), grass_common.wgsl and grass_blade.wgsl (group 1).
//
// Ported from bevy_grass v0.20.3 (src/shaders/grass_shadow.wesl;
// third_party/bevy-grass). MIT License, Copyright (c) 2026 Gediminas
// Mikelenas: the full notice is in grass_common.wgsl and
// third_party/bevy-grass/LICENSE.

@vertex
fn vs_shadow(@builtin(vertex_index) vi: u32, @builtin(instance_index) ii: u32) -> @builtin(position) vec4<f32> {
  var e: GrassEye;
  e.position = vec3<f32>(0.0);
  e.toward = grass.sun.xyz;
  e.orthographic = true;
  e.view_proj = shadow_pass.view_proj;
  e.pixel = 0.0;
  e.previous = false;
  let v = grass_vertex(grass_ref(vi, ii), e);
  return shadow_pass.view_proj * vec4<f32>(v.world, 1.0);
}

@fragment
fn fs_shadow(@builtin(position) clip: vec4<f32>) -> @location(0) vec4<f32> {
  return vec4<f32>(clip.z, 0.0, 0.0, 1.0);
}

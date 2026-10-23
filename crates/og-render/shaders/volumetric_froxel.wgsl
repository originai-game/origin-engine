// The froxel grid of volumetric.wgsl (full tier): inject, integrate, apply.

@group(1) @binding(1) var froxel_out: texture_storage_3d<rgba16float, write>;

@compute @workgroup_size(8, 8, 1)
fn inject(@builtin(global_invocation_id) id: vec3<u32>) {
  let grid = vec3<u32>(vol.grid.xyz);
  if (id.x >= grid.x || id.y >= grid.y || id.z >= grid.z) { return; }
  let uv = (vec2<f32>(id.xy) + 0.5) / vol.grid.xy;
  let dir = view_ray(uv);
  let depth = slice_depth((f32(id.z) + 0.5) / vol.grid.z);
  let p = frame.view_pos.xyz + dir * depth / max(dot(dir, frame.view_forward.xyz), 1e-3);
  let sigma = vol_density(p);
  var light = vec3<f32>(0.0);
  if (sigma > 0.0) {
    light = sun_in(p, dir) + locals_in(p, dir, uv * vol.extent.zw);
  }
  textureStore(froxel_out, id, vec4<f32>(vol.scatter.rgb * sigma * light, sigma));
}

@group(1) @binding(2) var froxel_in: texture_3d<f32>;
@group(1) @binding(3) var froxel_sum: texture_storage_3d<rgba16float, write>;

@compute @workgroup_size(8, 8, 1)
fn integrate(@builtin(global_invocation_id) id: vec3<u32>) {
  let grid = vec3<u32>(vol.grid.xyz);
  if (id.x >= grid.x || id.y >= grid.y) { return; }
  let uv = (vec2<f32>(id.xy) + 0.5) / vol.grid.xy;
  let stretch = 1.0 / max(dot(view_ray(uv), frame.view_forward.xyz), 1e-3);
  var acc = vec3<f32>(0.0);
  var transmittance = 1.0;
  var near = 0.0;
  for (var z = 0u; z < grid.z; z++) {
    let far = slice_depth(f32(z + 1u) / vol.grid.z);
    let thick = (far - near) * stretch;
    near = far;
    let s = textureLoad(froxel_in, vec3<i32>(vec3<u32>(id.xy, z)), 0);
    let tr = exp(-s.a * thick);
    acc += transmittance * s.rgb * select(thick, (1.0 - tr) / s.a, s.a > 1e-5);
    transmittance *= tr;
    textureStore(froxel_sum, vec3<u32>(id.xy, z), vec4<f32>(acc, transmittance));
  }
}

@group(1) @binding(5) var froxel_tex: texture_3d<f32>;
@group(1) @binding(6) var froxel_sampler: sampler;

@fragment
fn fs_froxel(in: VolOut) -> @location(0) vec4<f32> {
  let d = min(pixel_depth(in.pos.xy), vol.extent.x);
  let uv = in.pos.xy / vol.extent.zw;
  let f = sqrt(max(d, 0.0) / vol.extent.x);
  let z = f - 0.5 / vol.grid.z;
  let s = textureSampleLevel(froxel_tex, froxel_sampler, vec3<f32>(uv, max(z, 0.5 / vol.grid.z)), 0.0);
  // Nearer than the first slice's far edge: that slice's share of it.
  let k = clamp(f * vol.grid.z, 0.0, 1.0);
  return mix(vec4<f32>(0.0, 0.0, 0.0, 1.0), s, k);
}


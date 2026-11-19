// A packed depth prepass (common.wgsl `prepass_pack`) written back into
// its own targets, one fullscreen pass each (prepass_form.rs): a device
// without multiple render targets draws the prepass into one texel.

@group(0) @binding(0) var packed: texture_2d<u32>;

fn packed_at(clip: vec4<f32>) -> vec4<u32> {
  return textureLoad(packed, vec2<i32>(clip.xy), 0);
}

// The `gbuffer` target: the octahedral normal, then the velocity.
@fragment
fn fs_gbuffer(in: VsOut) -> @location(0) vec4<f32> {
  let p = packed_at(in.pos);
  return vec4<f32>(unpack2x16float(p.x), unpack2x16float(p.y));
}

// The `lindepth` target: the view depth and the reflectivity's bits.
@fragment
fn fs_lindepth(in: VsOut) -> @location(0) vec4<f32> {
  let p = packed_at(in.pos);
  return vec4<f32>(bitcast<f32>(p.z), bitcast<f32>(p.w), 0.0, 1.0);
}

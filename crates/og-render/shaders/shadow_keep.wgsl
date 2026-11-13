// A shadow layer's kept static casters put back (shadow_keep.rs): the
// first draw of a pass that draws the dynamic casters over them. It writes
// the kept value and the depth the casters after it are tested against.

@group(0) @binding(0) var kept: texture_2d<f32>;
@group(0) @binding(1) var kept_depth: texture_depth_2d;

struct KeptOut {
  @location(0) value: vec4<f32>,
  @builtin(frag_depth) depth: f32,
};

// A cascade's value is its casters' depth (shadow_pass.wgsl): one read.
@fragment
fn fs_cascade(in: VsOut) -> KeptOut {
  let value = textureLoad(kept, vec2<i32>(in.pos.xy), 0);
  return KeptOut(value, value.r);
}

// A local layer's two channels are distances, not its depth
// (`local_value`): the depth is kept beside them.
@fragment
fn fs_local(in: VsOut) -> KeptOut {
  let at = vec2<i32>(in.pos.xy);
  return KeptOut(textureLoad(kept, at, 0), textureLoad(kept_depth, at, 0));
}

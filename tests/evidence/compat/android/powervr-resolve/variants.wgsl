
struct VNoDepth { @location(0) color: vec4<f32> };

@fragment
fn v_nodepth(in: MeshletResolveOut) -> VNoDepth {
  let r = meshlet_record(in.clip, in.row);
  if (!r.ok) { discard; }
  var o: VNoDepth;
  hook_front = 1.0;
  analytic_uv = r.lit_uv;
  o.color = lit_fragment(r.v);
  return o;
}

@fragment
fn v_constlit(in: MeshletResolveOut) -> MeshletResolveLit {
  let r = meshlet_record(in.clip, in.row);
  if (!r.ok) { discard; }
  var o: MeshletResolveLit;
  o.depth = r.depth;
  o.color = vec4<f32>(r.v.normal, 1.0);
  return o;
}

@fragment
fn v_nodiscard(in: MeshletResolveOut) -> MeshletResolveLit {
  let r = meshlet_record(in.clip, in.row);
  var o: MeshletResolveLit;
  o.depth = r.depth;
  hook_front = 1.0;
  analytic_uv = r.lit_uv;
  o.color = lit_fragment(r.v);
  return o;
}

@fragment
fn v_noanalytic(in: MeshletResolveOut) -> MeshletResolveLit {
  let r = meshlet_record(in.clip, in.row);
  if (!r.ok) { discard; }
  var o: MeshletResolveLit;
  o.depth = r.depth;
  hook_front = 1.0;
  o.color = lit_fragment(r.v);
  return o;
}

@fragment
fn v_synth(in: MeshletResolveOut) -> MeshletResolveLit {
  var v: VertexOut;
  v.clip = in.clip;
  v.world_pos = vec3<f32>(in.clip.xy, 1.0);
  v.normal = vec3<f32>(0.0, 1.0, 0.0);
  v.uv = in.clip.xy * 0.001;
  v.color = vec4<f32>(1.0);
  v.tangent = vec4<f32>(1.0, 0.0, 0.0, 1.0);
  v.affine = vec3<f32>(v.uv, 1.0);
  var o: MeshletResolveLit;
  o.depth = 0.5;
  hook_front = 1.0;
  o.color = lit_fragment(v);
  return o;
}

fn v_synthv(in: MeshletResolveOut) -> VertexOut {
  var v: VertexOut;
  v.clip = in.clip;
  v.world_pos = vec3<f32>(in.clip.xy, 1.0);
  v.normal = vec3<f32>(0.0, 1.0, 0.0);
  v.uv = in.clip.xy * 0.001;
  v.color = vec4<f32>(1.0);
  v.tangent = vec4<f32>(1.0, 0.0, 0.0, 1.0);
  v.affine = vec3<f32>(v.uv, 1.0);
  return v;
}

// A discard on a storage read, no record.
@fragment
fn v_discard_only(in: MeshletResolveOut) -> MeshletResolveLit {
  let px = u32(in.clip.y) * meshlet_screen.x + u32(in.clip.x);
  if (meshlet_attrs[px / 4u][px % 4u] == 0u) { discard; }
  var o: MeshletResolveLit;
  o.depth = 0.5;
  hook_front = 1.0;
  o.color = lit_fragment(v_synthv(in));
  return o;
}

// The record read (its globals' writes included) but its vertex unused.
@fragment
fn v_record_unused(in: MeshletResolveOut) -> MeshletResolveLit {
  let r = meshlet_record(in.clip, in.row);
  if (!r.ok) { discard; }
  var o: MeshletResolveLit;
  o.depth = r.depth;
  hook_front = 1.0;
  o.color = lit_fragment(v_synthv(in));
  return o;
}

// No discard at all: the record's vertex, the shading unconditional.
@fragment
fn v_record_nobranch(in: MeshletResolveOut) -> MeshletResolveLit {
  let r = meshlet_record(in.clip, in.row);
  var o: MeshletResolveLit;
  o.depth = r.depth;
  hook_front = 1.0;
  o.color = lit_fragment(r.v);
  return o;
}

// A storage-read vertex without the record's helper: one row, straight loads.
@fragment
fn v_flat_vertex(in: MeshletResolveOut) -> MeshletResolveLit {
  let px = u32(in.clip.y) * meshlet_screen.x + u32(in.clip.x);
  let a = meshlet_attrs[px];
  var v = v_synthv(in);
  v.world_pos = bitcast<vec3<f32>>(a.xyz);
  var o: MeshletResolveLit;
  o.depth = 0.5;
  hook_front = 1.0;
  o.color = lit_fragment(v);
  return o;
}

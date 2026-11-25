// Native Tier::Full bindless material table entry points (`shading/bindless.rs`,
// `Caps::BINDLESS_TEXTURES | Caps::VERTEX_STORAGE`):
//   - Group 1 binds ONE shared material table (`bindless_materials`), ONE
//     sampled texture binding array (`bindless_textures[1024]`), ONE sampler,
//     and ONE per-instance material index buffer (`bindless_instance_materials`).
//   - `vs_bindless` looks up `mat_id = bindless_instance_materials[instance_index]`,
//     eliminating per-material bind-group switches and enabling multi-material
//     `multi_draw_indexed_indirect` coalescing.
//   - `fs_bindless` evaluates all screen-space derivatives in uniform control
//     flow up front (`ANALYTIC = true`, `DISCARD = false`) before loading the
//     per-instance material record and calling `lit_fragment`.

fn bindless_sample(tex_idx: u32, slot: u32, uv: vec2<f32>) -> vec4<f32> {
  let at = material_uv(slot, uv);
  let gx = analytic_grad(slot, uv, analytic_uv.xy);
  let gy = analytic_grad(slot, uv, analytic_uv.zw);
  return textureSampleGrad(bindless_textures[tex_idx], material_sampler, at, gx, gy);
}

@vertex
fn vs_bindless(
  v: VertexIn,
  i: InstanceIn,
  k: InstanceFxIn,
  @builtin(instance_index) inst: u32,
) -> VertexOut {
  var o = lit_vertex(instance_matrix(i.m0, i.m1, i.m2, i.m3), v);
  o.look = k.look;
  o.mat_id = bindless_instance_materials[inst];
  return o;
}

@fragment
fn fs_bindless(
  in: VertexOut,
  @builtin(front_facing) front_facing: bool,
) -> @location(0) vec4<f32> {
  // Compute all derivatives in guaranteed uniform control flow before
  // indexing the per-instance material record (`ANALYTIC = true`).
  analytic_uv = vec4<f32>(dpdxFine(in.uv), dpdyFine(in.uv));
  analytic_dx = dpdxFine(in.world_pos);
  analytic_dy = dpdyFine(in.world_pos);
  analytic_local_dx = dpdxFine(in.local_pos);
  analytic_local_dy = dpdyFine(in.local_pos);
  analytic_dnx = dpdxFine(in.normal);
  analytic_dny = dpdyFine(in.normal);
  let bm = bindless_materials[in.mat_id];
  material = bm.factors;
  bindless_tex0 = bm.tex0;
  bindless_tex1 = bm.tex1;
  hook_front = select(-1.0, 1.0, front_facing);
  return lit_fragment(in);
}

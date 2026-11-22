// The scattered copies in the shadow passes: depth from a cascade, or
// distance / range from a local light, like shadow.wgsl, shrunk and
// crossfaded as the lit pass draws them so their shadows match. Prepended by
// noise.wgsl, material.wgsl (group 1: the alpha cut-off), foliage.wgsl,
// scatter_common.wgsl (group 2) and shadow_pass.wgsl (group 0, the shadow
// pass's slot).

struct ScatterShadowOut {
  @builtin(position) clip: vec4<f32>,
  @location(0) world: vec3<f32>,
  @location(1) uv: vec2<f32>,
  @location(2) @interpolate(flat, either) fade: f32,
};

@vertex
fn vs_shadow(v: ScatterVertex, i: ScatterInstance) -> ScatterShadowOut {
  return scatter_shadow_vertex(v, i, SCATTER_RIGID, false);
}

// A foliage mesh's copy (pipelines.rs FOLIAGE_KIND), its leaves grown and
// bent as the lit pass draws them (scatter.wgsl scatter_vertex).
@vertex
fn vs_shadow_foliage(v: ScatterVertex, i: ScatterInstance, f: ScatterFoliage) -> ScatterShadowOut {
  return scatter_shadow_vertex(v, i, f, true);
}

fn scatter_shadow_vertex(v: ScatterVertex, i: ScatterInstance, f: ScatterFoliage, foliage: bool) -> ScatterShadowOut {
  let size = scatter_scale(i.m3);
  let model = scatter_shrunk(instance_matrix(i.m0, i.m1, i.m2, i.m3), size);
  let scale = length(i.m1) * size;
  var rest = v.position;
  var sway: vec3<f32>;
  if (foliage) {
    rest = meshlet_grown(rest, f.leaf, scatter_leaf(f, model).leaf.kept);
    sway = scatter_foliage_sway(rest, f, model, i.m3, scale, foliage_wind_unpack(shadow_pass.foliage_wind), false);
  } else {
    sway = scatter_sway(rest, i.m3, scale, false);
  }
  let world = model * vec4<f32>(rest, 1.0) + vec4<f32>(sway, 0.0);
  var o: ScatterShadowOut;
  o.fade = scatter_near(i.m3);
  o.clip = select(SCATTER_GONE, shadow_project(world, mul_point(shadow_pass.view_proj, world)), size > 0.0 && o.fade > 0.0);
  o.world = world.xyz;
  o.uv = v.uv;
  return o;
}

@fragment
fn fs_shadow(in: ScatterShadowOut, @builtin(front_facing) front: bool) -> @location(0) vec4<f32> {
  let dx = dpdxFine(in.world);
  let dy = dpdyFine(in.world);
  let base_uv = material_uv(UV_MAP, in.uv);
  let a = material.base_color.a * textureSample(base_color_tex, material_sampler, base_uv).a;
  if ((material.emissive.w >= 0.0 && a < material.emissive.w) || dither_cut(scatter_hidden(in.fade), in.clip.xy, shadow_pass.dither.x)) {
    discard;
  }
  return shadow_depth(in.world, in.clip, dx, dy, front);
}

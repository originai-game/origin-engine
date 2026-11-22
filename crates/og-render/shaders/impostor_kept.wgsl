// The main camera's lit pass over the prepass's own depth (single-sampled,
// scatter.rs EQUAL_KIND past IMPOSTOR_KIND): the prepass found each pixel's
// surface (its view depth in `imp_view_depth`), so the lit stage takes it
// instead of searching the frames again, and writes no depth.
@group(2) @binding(5) var imp_view_depth: texture_storage_2d<rg32float, read>;

// The card's colour where the prepass kept the surface it stands for: that
// point rebuilt from the view depth along the pixel's ray, each frame read
// where it sees the point, and the pixel kept only where the frames cover it
// at that depth (another surface won the prepass there otherwise).
@fragment
fn fs_impostor_kept(in: ImpostorOut) -> @location(0) vec4<f32> {
  imp_set_level(in.uv01.xy);
  let t = textureLoad(imp_view_depth, vec2<i32>(in.clip.xy)).r;
  let ortho = frame.view_pos.w > 0.5;
  let dir = select(normalize(in.world - frame.view_pos.xyz), frame.view_forward.xyz, ortho);
  let along = dot(in.world - frame.view_pos.xyz, frame.view_forward.xyz);
  let world = in.world + dir * (t - along) / max(dot(dir, frame.view_forward.xyz), 1e-4);
  // In the model's space about its rest pose (the frames' depths are the
  // unswayed model's), in radii.
  let m3 = mat3x3<f32>(in.m0, in.m1, in.m2);
  let x = transpose(m3) * (world - (in.world - in.rest) - in.centre) / in.root_extent.w;
  let uv01 = vec4<f32>(imp_frame_uv(x, in.frames.x), imp_frame_uv(x, in.frames.y));
  let uv2 = imp_frame_uv(x, in.frames.z);
  let c = imp_coverage(uv01, uv2, in.frames, in.weights);
  let total = max(c.wa.x + c.wa.y + c.wa.z, 1e-5);
  let d = vec3<f32>(
    textureSampleLevel(normal_tex, material_sampler, c.a0, imp_level).a,
    textureSampleLevel(normal_tex, material_sampler, c.a1, imp_level).a,
    textureSampleLevel(normal_tex, material_sampler, c.a2, imp_level).a,
  ) * 2.0 - 1.0;
  let pd = vec3<f32>(dot(x, imp_frame_dir(in.frames.x)), dot(x, imp_frame_dir(in.frames.y)), dot(x, imp_frame_dir(in.frames.z)));
  let off = abs(dot(d - pd, c.wa)) / total;
  if (is_sky_depth(t) || c.alpha < imp_cutoff() || off > IMP_AGREE || dot(x, x) > 1.21 || dither_cut(in.fade, in.clip.xy, frame.drawn.w)) {
    discard;
  }
  // The prepass cut the fallen leaves; this pass draws only what it kept.
  let surface = imp_surface_of(in, c, world);
  return imp_shade(in, imp_leaf_surface(surface, imp_leaf(in, surface)), 1.0);
}

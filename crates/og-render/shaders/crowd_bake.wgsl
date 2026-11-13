// A crowd impostor's bake (crowd_impostor.rs, crowd.rs **Impostors**): a
// part's full mesh posed by the impostor's clip and drawn once per tile of
// the atlas, instance `frame * views + view`: tile (view, frame) shows the
// model at frame `frame` of the frames the atlas holds, seen
// orthographically from `view` of the directions around the vertical at
// the bake's elevation, its bounding sphere filling the tile. Prepended
// with noise.wgsl, material.wgsl and material_maps.wgsl (group 1: the
// part's material) and the palette (group 2, skin_storage.wgsl).
//
// Both targets are premultiplied by coverage (the background is 0), so the
// multisample resolve and the mip chain average them as coverage does:
//   albedo  rgb: the part's colour detail (its base colour map times its
//           vertex colour, not its material's colour), a: coverage
//   data    xy: the normal in the view's basis (octahedral), z: the depth
//           towards the viewer in sphere radii, w: the part's index

struct Bake {
  // x: the clip's first slot in the baked buffer, y: its joints, z: its
  // frames, w: the part's index
  clip: vec4<u32>,
  // x: views around the vertical, y: frames the atlas holds
  tiles: vec4<u32>,
  // xyz: the sphere's centre, w: its radius (the model's space)
  sphere: vec4<f32>,
  // x: the elevation's sine, y: its cosine
  view: vec4<f32>,
};

@group(0) @binding(0) var<uniform> bake: Bake;

struct BakeVertex {
  @location(0) position: vec3<f32>,
  @location(1) normal: vec3<f32>,
  @location(2) uv: vec2<f32>,
  @location(3) color: vec4<f32>,
};

struct BakeSkin {
  @location(9) joints: vec4<u32>,
  @location(10) weights: vec4<f32>,
};

struct BakeOut {
  @builtin(position) clip: vec4<f32>,
  @location(0) uv: vec2<f32>,
  @location(1) color: vec4<f32>,
  // The view's basis: the normal in it, and where in the tile it falls.
  @location(2) normal: vec3<f32>,
  @location(3) tile: vec3<f32>,
};

// The direction tile column `view` looks from (from the centre towards the
// viewer), and its right and up (impostor_common.wgsl imp_right's rule).
fn bake_basis(view: u32) -> mat3x3<f32> {
  let a = 6.283185307 * f32(view) / f32(bake.tiles.x);
  let d = vec3<f32>(bake.view.y * sin(a), bake.view.x, bake.view.y * cos(a));
  let r = normalize(cross(vec3<f32>(0.0, 1.0, 0.0), d));
  return mat3x3<f32>(r, cross(d, r), d);
}

@vertex
fn vs_bake(@builtin(instance_index) tile: u32, v: BakeVertex, s: BakeSkin) -> BakeOut {
  let views = bake.tiles.x;
  let frames = bake.tiles.y;
  let view = tile % views;
  let frame = tile / views;
  // The atlas's frames spread over the clip, which loops (its last frame
  // is its first): frame k at k / frames of the way.
  let at = f32(frame) / f32(frames) * f32(bake.clip.z - 1u);
  let k = min(u32(floor(at)), bake.clip.z - 2u);
  let palette = vec4<f32>(f32(bake.clip.x + k * bake.clip.y), f32(bake.clip.y), 0.0, -1.0 - clamp(at - f32(k), 0.0, 1.0));
  var skin = mat4x4<f32>(vec4<f32>(0.0), vec4<f32>(0.0), vec4<f32>(0.0), vec4<f32>(0.0));
  for (var i = 0u; i < 4u; i++) {
    skin += palette_joint(palette, s.joints[i]) * s.weights[i];
  }
  let p = (skin * vec4<f32>(v.position, 1.0)).xyz - bake.sphere.xyz;
  let n = normalize((skin * vec4<f32>(v.normal, 0.0)).xyz + vec3<f32>(0.0, 1e-9, 0.0));
  let b = bake_basis(view);
  let x = dot(p, b[0]) / bake.sphere.w;
  let y = dot(p, b[1]) / bake.sphere.w;
  let z = dot(p, b[2]) / bake.sphere.w;
  var o: BakeOut;
  o.clip = vec4<f32>(
    -1.0 + 2.0 * (f32(view) + 0.5 + 0.5 * x) / f32(views),
    1.0 - 2.0 * (f32(frame) + 0.5 - 0.5 * y) / f32(frames),
    clamp(0.5 - 0.5 * z, 0.0, 1.0),
    1.0,
  );
  o.uv = v.uv;
  o.color = v.color;
  o.normal = vec3<f32>(dot(n, b[0]), dot(n, b[1]), dot(n, b[2]));
  o.tile = vec3<f32>(x, y, z);
  return o;
}

struct BakeTargets {
  @location(0) albedo: vec4<f32>,
  @location(1) data: vec4<f32>,
};

@fragment
fn fs_bake(in: BakeOut) -> BakeTargets {
  // Nothing spills into the next tile.
  if (any(abs(in.tile.xy) > vec2<f32>(1.0))) { discard; }
  let tex = textureSample(base_color_tex, material_sampler, material_uv(UV_MAP, in.uv));
  let alpha = tex.a * in.color.a * material.base_color.a;
  if (material.emissive.w >= 0.0 && alpha < material.emissive.w) { discard; }
  var o: BakeTargets;
  o.albedo = vec4<f32>(tex.rgb * in.color.rgb, 1.0);
  o.data = vec4<f32>(oct_encode(normalize(in.normal)), in.tile.z, f32(bake.clip.w));
  return o;
}

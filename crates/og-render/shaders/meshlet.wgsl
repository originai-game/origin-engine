// The meshlet path's vertex stages and tile-classified visibility-buffer
// resolve (meshlet_gpu.rs), after pbr.wgsl and pbr_prepass.wgsl in the
// standard lit module.
//
// Hardware path (`vs_meshlet`, `vs_prepass_meshlet`): 64 x 3 vertices per
// visible cluster pulled from group 3 bindings 8..14, 17, 18.
//
// Visibility-buffer resolve (`vs_meshlet_resolve`, `fs_meshlet_resolve`,
// `fs_prepass_meshlet_resolve`): draws only the 8x8 tiles classified active for
// each soft batch via indirect draws (`vis_tiles`, `vis_row`), pulls the
// winning triangle directly from the 8 B/px visibility buffer (`vis_vbuf`) +
// geometry pools, computes exact perspective-correct barycentrics and analytic
// UV/world/local/normal derivatives (fixing B3 overflow & B8 specular AA /
// weathering bend), and shades via `lit_fragment` / `prepass_targets`.

// meshlet_sway.wgsl's `MeshletDraw`.
@group(3) @binding(8) var<uniform> meshlet_draw: MeshletDraw;
// The wind field's lean (wind_field.rs; meshlet_sway.wgsl).
@group(3) @binding(18) var meshlet_wind_tex: texture_2d<f32>;
// The culled instances' transforms (instances.rs `GpuTransform`, 4 x vec4).
@group(3) @binding(9) var<storage, read> meshlet_instances: array<vec4<f32>>;
// Work items: hardware draws bind a dynamic window; resolve binds the whole buffer.
@group(3) @binding(10) var<storage, read> meshlet_items: array<vec2<u32>>;
// Per cluster 3 x vec4: (first index, triangles, level, foliage region + 1), sphere, cone.
@group(3) @binding(11) var<storage, read> meshlet_clusters: array<vec4<u32>>;
@group(3) @binding(12) var<storage, read> meshlet_indices: array<u32>;
// Hardware draw mesh vertices (scene Vertex: 16 floats).
@group(3) @binding(13) var<storage, read> meshlet_vertices: array<vec4<f32>>;
// The instances' previous transforms, at the same slots (read by prepass only).
@group(3) @binding(14) var<storage, read> meshlet_previous: array<vec4<f32>>;
// The full vertex pool (all clustered meshes' vertices followed by foliage regions).
@group(3) @binding(17) var<storage, read> meshlet_foliage: array<vec4<f32>>;

// Visibility-buffer resolve bindings (vis.rs):
// (width, height, soft_first, 0).
@group(3) @binding(15) var<uniform> meshlet_screen: vec4<u32>;
// Compact 8 B/px visibility buffer: depth words [0..wh), id words [wh..2*wh).
@group(3) @binding(16) var<storage, read> vis_vbuf: array<u32>;
// Compact tile lists: [0..2*soft_rows) = (start, count), tail = (tx | ty << 16).
@group(3) @binding(19) var<storage, read> vis_tiles: array<u32>;
// Current soft batch row (dynamic uniform offset per draw).
@group(3) @binding(20) var<uniform> vis_row: vec4<u32>;

struct MeshletVertex {
  ok: bool,
  v: VertexIn,
  model: mat4x4<f32>,
  look: vec4<f32>,
  slot: u32,
  region: u32,
  index: u32,
  rest: vec3<f32>,
  leaf: MeshletLeaf,
};

fn meshlet_rows(region: u32, index: u32) -> array<vec4<f32>, 3> {
  let at = meshlet_sway_row(region, index);
  return array<vec4<f32>, 3>(meshlet_foliage[region - 1u], meshlet_foliage[at], meshlet_foliage[at + 1u]);
}

fn meshlet_transform(t0: vec4<f32>, t1: vec4<f32>, t2: vec4<f32>) -> mat4x4<f32> {
  return instance_matrix(t0.xyz, vec3<f32>(t0.w, t1.xy), vec3<f32>(t1.zw, t2.x), t2.yzw);
}

fn meshlet_level_color(level: u32) -> f32 {
  var c = vec3<f32>(0.9, 0.08, 0.08);
  if (level == 0u) { c = vec3<f32>(0.1, 0.85, 0.15); }
  else if (level == 1u) { c = vec3<f32>(0.95, 0.85, 0.1); }
  else if (level == 2u) { c = vec3<f32>(1.0, 0.45, 0.05); }
  let q = vec3<u32>(c * 255.0 + 0.5);
  return f32((q.x << 16u) | (q.y << 8u) | q.z);
}

fn meshlet_vertex(vi: u32, ii: u32) -> MeshletVertex {
  var r: MeshletVertex;
  let item = meshlet_items[ii];
  let c = meshlet_clusters[item.y * 3u];
  if (vi / 3u >= c.y) {
    r.ok = false;
    return r;
  }
  r.ok = true;
  let index = meshlet_indices[c.x + vi];
  let a = meshlet_vertices[index * 4u];
  let b = meshlet_vertices[index * 4u + 1u];
  r.v.position = a.xyz;
  r.v.normal = vec3<f32>(a.w, b.xy);
  r.v.uv = b.zw;
  r.v.color = meshlet_vertices[index * 4u + 2u];
  r.v.tangent = meshlet_vertices[index * 4u + 3u];
  let t0 = meshlet_instances[item.x * 4u];
  let t1 = meshlet_instances[item.x * 4u + 1u];
  let t2 = meshlet_instances[item.x * 4u + 2u];
  r.model = meshlet_transform(t0, t1, t2);
  r.slot = item.x;
  r.look = meshlet_instances[item.x * 4u + 3u];
  r.region = c.w;
  r.index = index;
  r.rest = r.v.position;
  r.leaf = MeshletLeaf(SeasonLeaf(1.0, 0.0, 0.0), 0.0);
  if (c.w != 0u) {
    let f = meshlet_rows(c.w, index);
    r.leaf = meshlet_leaf(meshlet_foliage[c.w], f[2], r.model, meshlet_draw.wind[0].origin.xyz, meshlet_draw.leaf_year);
    let scale = meshlet_level_scale(meshlet_foliage[meshlet_scale_row(c.w, c.z)], c.z);
    r.rest = meshlet_grown(r.rest, f[2], r.leaf.leaf.kept * scale);
    r.v.position = r.rest + meshlet_sway(r.rest, f[0], f[1], f[2], r.model, meshlet_draw.wind[0], foliage_sway(r.look.x), false);
  }
  if (meshlet_draw.flags.x != 0u || frame.drawn.y != 0u) {
    r.look.y = 0.8;
    r.look.z = meshlet_level_color(c.z);
  }
  return r;
}

@vertex
fn vs_meshlet(@builtin(vertex_index) vi: u32, @builtin(instance_index) ii: u32) -> VertexOut {
  let p = meshlet_vertex(vi, ii);
  if (!p.ok) {
    var o: VertexOut;
    o.clip = vec4<f32>(0.0);
    return o;
  }
  var v = p.v;
  if (p.region != 0u) {
    let f = meshlet_rows(p.region, p.index);
    v.normal = foliage_normal(v.normal, f[2].xyz, f[0].xyz, material.coat.w);
  }
  var o = lit_vertex(p.model, v);
  o.look = p.look;
  if (p.region != 0u) {
    o.season = meshlet_leaf_color(meshlet_foliage[p.region + 1u], meshlet_foliage[p.region + 2u], p.leaf);
    let f = meshlet_rows(p.region, p.index);
    let axis = meshlet_crown_axis(p.v.normal, f[2], f[0], meshlet_foliage[p.region]);
    let sun = meshlet_model_dir(p.model, frame.light_dir.xyz);
    o.crown = meshlet_crown(p.rest, axis, sun, f[0], meshlet_foliage[p.region + 3u]);
  }
  return o;
}

@vertex
fn vs_prepass_meshlet(@builtin(vertex_index) vi: u32, @builtin(instance_index) ii: u32) -> PrepassOut {
  let p = meshlet_vertex(vi, ii);
  if (!p.ok) {
    var o: PrepassOut;
    o.clip = vec4<f32>(0.0);
    return o;
  }
  let q = p.slot * 4u;
  let prev = meshlet_transform(meshlet_previous[q], meshlet_previous[q + 1u], meshlet_previous[q + 2u]);
  var before = p.rest;
  if (p.region != 0u) {
    let f = meshlet_rows(p.region, p.index);
    before += meshlet_sway(p.rest, f[0], f[1], f[2], prev, meshlet_draw.wind[1], foliage_sway(meshlet_previous[q + 3u].x), true);
  }
  return prepass_vertex_from(p.model, prev, p.v, before);
}

// ── Tile-classified visibility-buffer resolve ──────────────────────────────

struct MeshletResolveOut {
  @builtin(position) clip: vec4<f32>,
  @location(0) @interpolate(flat, either) row: u32,
};

@vertex
fn vs_meshlet_resolve(@builtin(vertex_index) vi: u32, @builtin(instance_index) ii: u32) -> MeshletResolveOut {
  let row = vis_row.x;
  let base = vis_tiles[row * 2u];
  let packed = vis_tiles[base + ii];
  let tx = packed & 0xffffu;
  let ty = packed >> 16u;
  let w = f32(meshlet_screen.x);
  let h = f32(meshlet_screen.y);
  let x0 = f32(tx * 8u);
  let y0 = f32(ty * 8u);
  let x1 = min(x0 + 8.0, w);
  let y1 = min(y0 + 8.0, h);
  let nx0 = x0 / w * 2.0 - 1.0;
  let nx1 = x1 / w * 2.0 - 1.0;
  let ny0 = 1.0 - y0 / h * 2.0;
  let ny1 = 1.0 - y1 / h * 2.0;
  var pos = vec2<f32>(nx0, ny0);
  if (vi == 1u) { pos = vec2<f32>(nx0, ny1); }
  else if (vi == 2u || vi == 4u) { pos = vec2<f32>(nx1, ny1); }
  else if (vi == 5u) { pos = vec2<f32>(nx1, ny0); }
  var o: MeshletResolveOut;
  o.clip = vec4<f32>(pos, 0.0, 1.0);
  o.row = row;
  return o;
}

fn vis_screen_weights(clip: array<vec4<f32>, 3>, ndc: vec2<f32>) -> vec3<f32> {
  let p0 = clip[0].xy / clip[0].w;
  let p1 = clip[1].xy / clip[1].w;
  let p2 = clip[2].xy / clip[2].w;
  let area = (p1.x - p0.x) * (p2.y - p0.y) - (p1.y - p0.y) * (p2.x - p0.x);
  let inv = select(0.0, 1.0 / area, abs(area) > 1e-20);
  return vec3<f32>(
    (p1.x - ndc.x) * (p2.y - ndc.y) - (p1.y - ndc.y) * (p2.x - ndc.x),
    (p2.x - ndc.x) * (p0.y - ndc.y) - (p2.y - ndc.y) * (p0.x - ndc.x),
    (p0.x - ndc.x) * (p1.y - ndc.y) - (p0.y - ndc.y) * (p1.x - ndc.x),
  ) * inv;
}

fn vis_bary(clip: array<vec4<f32>, 3>, l: vec3<f32>) -> vec3<f32> {
  let q = l / vec3<f32>(clip[0].w, clip[1].w, clip[2].w);
  let sum = q.x + q.y + q.z;
  return select(vec3<f32>(1.0 / 3.0), q / sum, abs(sum) > 1e-20);
}

fn vis_leaf(local: u32, region: u32, now: mat4x4<f32>) -> MeshletLeaf {
  let at = meshlet_sway_row(region, local);
  return meshlet_leaf(meshlet_foliage[region], meshlet_foliage[at + 1u], now, meshlet_draw.wind[0].origin.xyz, meshlet_draw.leaf_year);
}

fn vis_rest_position(base: u32, local: u32, region: u32, level: u32, now: mat4x4<f32>) -> vec3<f32> {
  var p = meshlet_foliage[(base + local) * 4u].xyz;
  if (region == 0u) { return p; }
  let at = meshlet_sway_row(region, local);
  let scale = meshlet_level_scale(meshlet_foliage[meshlet_scale_row(region, level)], level);
  return meshlet_grown(p, meshlet_foliage[at + 1u], vis_leaf(local, region, now).leaf.kept * scale);
}

struct VisPull {
  ok: bool,
  front: f32,
  depth: f32,
  v: VertexOut,
  lit_uv: vec4<f32>,
  object: u32,
  // Triangle & barycentric data reused by prepass to compute previous world pos.
  inst: u32,
  region: u32,
  locals: vec3<u32>,
  rest: array<vec3<f32>, 3>,
  bary: vec3<f32>,
};

fn vis_pull(pos: vec4<f32>, row: u32) -> VisPull {
  var r: VisPull;
  r.ok = false;
  r.depth = 0.0;
  let w = meshlet_screen.x;
  let h = meshlet_screen.y;
  let px = u32(pos.x);
  let py = u32(pos.y);
  if (px >= w || py >= h) { return r; }
  let wh = w * h;
  let pixel = py * w + px;
  let id = vis_vbuf[wh + pixel];
  if (id == 0u) { return r; }
  let k = (id - 1u) >> 6u;
  let tri = (id - 1u) & 63u;
  let f = meshlet_items[meshlet_screen.z + k * 2u + 1u];
  if ((f.x & 0x0fffffffu) != row) { return r; }

  r.ok = true;
  r.depth = bitcast<f32>(vis_vbuf[pixel]);
  let e = meshlet_items[meshlet_screen.z + k * 2u];
  let c = meshlet_clusters[e.y * 3u];
  let q = e.x * 4u;
  let model = meshlet_transform(meshlet_instances[q], meshlet_instances[q + 1u], meshlet_instances[q + 2u]);
  var look = meshlet_instances[q + 3u];
  let sway_val = foliage_sway(look.x);
  let size = vec2<f32>(f32(w), f32(h));
  let ndc = vec2<f32>((f32(px) + 0.5) / size.x * 2.0 - 1.0, 1.0 - (f32(py) + 0.5) / size.y * 2.0);

  var locals: array<u32, 3>;
  var rest: array<vec3<f32>, 3>;
  var at: array<vec3<f32>, 3>;
  var clip: array<vec4<f32>, 3>;
  for (var j = 0u; j < 3u; j++) {
    let local = meshlet_indices[c.x + tri * 3u + j];
    locals[j] = local;
    let rp = vis_rest_position(f.y, local, c.w, c.z, model);
    rest[j] = rp;
    var sp = rp;
    if (c.w != 0u) {
      let fr = meshlet_rows(c.w, local);
      sp += meshlet_sway(rp, fr[0], fr[1], fr[2], model, meshlet_draw.wind[0], sway_val, false);
    }
    at[j] = sp;
    clip[j] = retro_clip(mul_point(frame.view_proj, model * vec4<f32>(sp, 1.0)));
  }

  let l = vis_screen_weights(clip, ndc);
  let lx = vis_screen_weights(clip, ndc + vec2<f32>(2.0 / size.x, 0.0));
  let ly = vis_screen_weights(clip, ndc - vec2<f32>(0.0, 2.0 / size.y));
  let b = vis_bary(clip, l);
  let bx = vis_bary(clip, lx);
  let by = vis_bary(clip, ly);

  var position = vec3<f32>(0.0);
  var normal = vec3<f32>(0.0);
  var dnorm_x = vec3<f32>(0.0);
  var dnorm_y = vec3<f32>(0.0);
  var uv = vec2<f32>(0.0);
  var color = vec4<f32>(0.0);
  var tangent = vec4<f32>(0.0);
  var dpos_x = vec3<f32>(0.0);
  var dpos_y = vec3<f32>(0.0);
  var duv_x = vec2<f32>(0.0);
  var duv_y = vec2<f32>(0.0);
  var affine = vec2<f32>(0.0);
  var daff_x = vec2<f32>(0.0);
  var daff_y = vec2<f32>(0.0);
  var dome = vec3<f32>(0.0);
  var season = vec4<f32>(0.0);
  var crown = vec2<f32>(0.0);
  let sun = meshlet_model_dir(model, frame.light_dir.xyz);

  for (var j = 0u; j < 3u; j++) {
    let vi = f.y + locals[j];
    let va = meshlet_foliage[vi * 4u];
    let vb = meshlet_foliage[vi * 4u + 1u];
    let vn = vec3<f32>(va.w, vb.xy);
    let dbx = bx[j] - b[j];
    let dby = by[j] - b[j];
    position += at[j] * b[j];
    dpos_x += at[j] * dbx;
    dpos_y += at[j] * dby;
    normal += vn * b[j];
    dnorm_x += vn * dbx;
    dnorm_y += vn * dby;
    uv += vb.zw * b[j];
    duv_x += vb.zw * dbx;
    duv_y += vb.zw * dby;
    affine += vb.zw * l[j];
    daff_x += vb.zw * (lx[j] - l[j]);
    daff_y += vb.zw * (ly[j] - l[j]);
    color += meshlet_foliage[vi * 4u + 2u] * b[j];
    tangent += meshlet_foliage[vi * 4u + 3u] * b[j];
    if (c.w != 0u) {
      let fr = meshlet_rows(c.w, locals[j]);
      dome += foliage_normal(vn, fr[2].xyz, fr[0].xyz, 1.0) * b[j];
      season += meshlet_leaf_color(meshlet_foliage[c.w + 1u], meshlet_foliage[c.w + 2u], vis_leaf(locals[j], c.w, model)) * b[j];
      let axis = meshlet_crown_axis(vn, fr[2], fr[0], meshlet_foliage[c.w]);
      crown += meshlet_crown(at[j], axis, sun, fr[0], meshlet_foliage[c.w + 3u]) * b[j];
    }
  }

  let world = (model * vec4<f32>(position, 1.0)).xyz;
  var n = world_normal(model, normal);
  let n_rx = world_normal(model, normal + dnorm_x);
  let n_ry = world_normal(model, normal + dnorm_y);
  if (c.w != 0u && material.coat.w > 0.0) {
    let dome_w = world_normal(model, dome);
    n = normalize(mix(n, dome_w, clamp(material.coat.w, 0.0, 1.0)));
  }
  let m3 = mat3x3<f32>(model[0].xyz, model[1].xyz, model[2].xyz);
  let hand = select(-1.0, 1.0, determinant(m3) >= 0.0);
  let tw = (model * vec4<f32>(tangent.xyz, 0.0)).xyz;
  let t_vec = select(vec3<f32>(0.0), normalize(tw), dot(tw, tw) > 1e-20);

  let p0 = clip[0].xy / clip[0].w;
  let e1 = clip[1].xy / clip[1].w - p0;
  let e2 = clip[2].xy / clip[2].w - p0;
  let back = e1.x * e2.y - e1.y * e2.x < 0.0;
  r.front = select(1.0, -1.0, back);

  r.object = outline_object(model[0].xyz, model[1].xyz, model[2].xyz, model[3].xyz, look) & 0xffffu;
  if (meshlet_draw.flags.x != 0u || frame.drawn.y != 0u) {
    look.y = 0.8;
    look.z = meshlet_level_color(c.z);
  }

  r.v.clip = pos;
  r.v.world_pos = world;
  r.v.normal = n;
  r.v.uv = uv;
  r.v.color = color;
  r.v.local_pos = position;
  r.v.tangent = vec4<f32>(t_vec, tangent.w * hand);
  r.v.affine = vec3<f32>(affine, 1.0);
  r.v.look = look;
  r.v.season = season;
  r.v.crown = crown;
  r.v.moves = 0u;

  analytic_uv = vec4<f32>(duv_x, duv_y);
  analytic_dx = (model * vec4<f32>(dpos_x, 0.0)).xyz;
  analytic_dy = (model * vec4<f32>(dpos_y, 0.0)).xyz;
  analytic_local_dx = dpos_x;
  analytic_local_dy = dpos_y;
  analytic_dnx = n_rx - n;
  analytic_dny = n_ry - n;
  let aff_share = clamp(frame.stylise.z, 0.0, 1.0);
  r.lit_uv = vec4<f32>(mix(duv_x, daff_x, aff_share), mix(duv_y, daff_y, aff_share));

  r.inst = e.x;
  r.region = c.w;
  r.locals = vec3<u32>(locals[0], locals[1], locals[2]);
  r.rest = rest;
  r.bary = b;
  return r;
}

struct MeshletResolveLit {
  @location(0) color: vec4<f32>,
  @builtin(frag_depth) depth: f32,
};

@fragment
fn fs_meshlet_resolve(in: MeshletResolveOut) -> MeshletResolveLit {
  let r = vis_pull(in.clip, in.row);
  if (!r.ok) { discard; }
  var o: MeshletResolveLit;
  o.depth = r.depth;
  hook_front = r.front;
  analytic_uv = r.lit_uv;
  o.color = lit_fragment(r.v);
  return o;
}

struct MeshletResolvePrepass {
  @location(0) gbuffer: vec4<f32>,
  @location(1) depth: vec4<f32>,
  @location(2) outline: vec4<f32>,
  @location(3) motion_depth: f32,
  @builtin(frag_depth) frag_depth: f32,
};

@fragment
fn fs_prepass_meshlet_resolve(in: MeshletResolveOut) -> MeshletResolvePrepass {
  let r = vis_pull(in.clip, in.row);
  if (!r.ok) { discard; }
  let q = r.inst * 4u;
  let prev = meshlet_transform(meshlet_previous[q], meshlet_previous[q + 1u], meshlet_previous[q + 2u]);
  let prev_sway = foliage_sway(meshlet_previous[q + 3u].x);
  var before = vec3<f32>(0.0);
  for (var j = 0u; j < 3u; j++) {
    var wp = r.rest[j];
    if (r.region != 0u) {
      let fr = meshlet_rows(r.region, r.locals[j]);
      wp += meshlet_sway(wp, fr[0], fr[1], fr[2], prev, meshlet_draw.wind[1], prev_sway, true);
    }
    before += wp * r.bary[j];
  }
  let prev_world = (prev * vec4<f32>(before, 1.0)).xyz;
  var p: PrepassOut;
  p.clip = in.clip;
  p.world_pos = r.v.world_pos;
  p.normal = r.v.normal;
  p.uv = r.v.uv;
  p.color = r.v.color;
  p.local_pos = r.v.local_pos;
  p.affine = r.v.affine;
  p.cur_clip = fx.view_proj_unjittered * vec4<f32>(r.v.world_pos, 1.0);
  p.prev_clip = mul_point(fx.prev_view_proj, vec4<f32>(prev_world, 1.0));
  p.object = r.object;
  hook_front = r.front;
  let t = prepass_targets(p);
  var o: MeshletResolvePrepass;
  o.gbuffer = t.gbuffer;
  o.depth = t.depth;
  o.outline = t.outline;
  o.motion_depth = t.motion_depth;
  o.frag_depth = r.depth;
  return o;
}

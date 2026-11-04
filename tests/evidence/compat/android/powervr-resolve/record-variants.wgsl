
fn rec_full(pos: vec4<f32>, row: u32) -> MeshletRecord {
  var r: MeshletRecord;


  let pixel = u32(pos.y) * meshlet_screen.x + u32(pos.x);
  let slot = meshlet_attrs[pixel / 4u][pixel % 4u];
  r.ok = false;
  r.depth = 1.0;
  if (slot == 0u) { return r; }
  let at = meshlet_screen.z / 4u + (slot - 1u) * meshlet_screen.w;
  let a = meshlet_attrs[at];
  let b = meshlet_attrs[at + 1u];
  let c = meshlet_attrs[at + 2u];
  if (c.z != row) { return r; }
  r.ok = true;
  r.depth = bitcast<f32>(a.w);
  r.v.clip = pos;
  r.v.world_pos = bitcast<vec3<f32>>(a.xyz);
  r.v.normal = meshlet_oct(unpack2x16snorm(b.x));
  r.v.uv = bitcast<vec2<f32>>(b.yz);
  r.v.color = unpack4x8unorm(b.w);

  r.v.local_pos = vec3<f32>(
    bitcast<f32>(meshlet_attrs[at + 4u].w),
    bitcast<f32>(meshlet_attrs[at + 6u].w),
    bitcast<f32>(meshlet_attrs[at + 7u].w),
  );
  r.v.tangent = vec4<f32>(meshlet_oct(unpack2x16snorm(c.x)), bitcast<f32>(c.y));
  if (r.v.tangent.w == 0.0) { r.v.tangent = vec4<f32>(0.0); }
  r.v.affine = vec3<f32>(r.v.uv, 1.0);
  r.v.look = bitcast<vec4<f32>>(meshlet_attrs[at + 3u]);
  r.prev_world = bitcast<vec3<f32>>(meshlet_attrs[at + 4u].xyz);

  analytic_uv = bitcast<vec4<f32>>(meshlet_attrs[at + 5u]);
  analytic_dx = bitcast<vec3<f32>>(meshlet_attrs[at + 6u].xyz);
  analytic_dy = bitcast<vec3<f32>>(meshlet_attrs[at + 7u].xyz);
  r.lit_uv = analytic_uv;
  if (meshlet_screen.w > 8u) {

    let lx = bitcast<vec4<f32>>(meshlet_attrs[at + 8u]);
    let ly = bitcast<vec4<f32>>(meshlet_attrs[at + 9u]);
    analytic_local_dx = lx.xyz;
    analytic_local_dy = ly.xyz;

    r.v.affine = vec3<f32>(lx.w, ly.w, 1.0);
    r.lit_uv = bitcast<vec4<f32>>(meshlet_attrs[at + 10u]);


    let d = meshlet_attrs[at + 11u];
    if (d.y != 0u && material.coat.w > 0.0) {
      r.v.normal = normalize(mix(normalize(r.v.normal), meshlet_oct(unpack2x16snorm(d.x)), clamp(material.coat.w, 0.0, 1.0)));
    }
  }
  r.object = c.w;
  return r;
}


@fragment
fn v_rec_full(in: MeshletResolveOut) -> MeshletResolveLit {
  let r = rec_full(in.clip, in.row);
  if (!r.ok) { discard; }
  var o: MeshletResolveLit;
  o.depth = r.depth;
  hook_front = 1.0;
  analytic_uv = r.lit_uv;
  o.color = lit_fragment(r.v);
  return o;
}

fn rec_noextra(pos: vec4<f32>, row: u32) -> MeshletRecord {
  var r: MeshletRecord;


  let pixel = u32(pos.y) * meshlet_screen.x + u32(pos.x);
  let slot = meshlet_attrs[pixel / 4u][pixel % 4u];
  r.ok = false;
  r.depth = 1.0;
  if (slot == 0u) { return r; }
  let at = meshlet_screen.z / 4u + (slot - 1u) * meshlet_screen.w;
  let a = meshlet_attrs[at];
  let b = meshlet_attrs[at + 1u];
  let c = meshlet_attrs[at + 2u];
  if (c.z != row) { return r; }
  r.ok = true;
  r.depth = bitcast<f32>(a.w);
  r.v.clip = pos;
  r.v.world_pos = bitcast<vec3<f32>>(a.xyz);
  r.v.normal = meshlet_oct(unpack2x16snorm(b.x));
  r.v.uv = bitcast<vec2<f32>>(b.yz);
  r.v.color = unpack4x8unorm(b.w);

  r.v.local_pos = vec3<f32>(
    bitcast<f32>(meshlet_attrs[at + 4u].w),
    bitcast<f32>(meshlet_attrs[at + 6u].w),
    bitcast<f32>(meshlet_attrs[at + 7u].w),
  );
  r.v.tangent = vec4<f32>(meshlet_oct(unpack2x16snorm(c.x)), bitcast<f32>(c.y));
  if (r.v.tangent.w == 0.0) { r.v.tangent = vec4<f32>(0.0); }
  r.v.affine = vec3<f32>(r.v.uv, 1.0);
  r.v.look = bitcast<vec4<f32>>(meshlet_attrs[at + 3u]);
  r.prev_world = bitcast<vec3<f32>>(meshlet_attrs[at + 4u].xyz);

  analytic_uv = bitcast<vec4<f32>>(meshlet_attrs[at + 5u]);
  analytic_dx = bitcast<vec3<f32>>(meshlet_attrs[at + 6u].xyz);
  analytic_dy = bitcast<vec3<f32>>(meshlet_attrs[at + 7u].xyz);
  r.lit_uv = analytic_uv;
  r.object = c.w;
  return r;
}


@fragment
fn v_rec_noextra(in: MeshletResolveOut) -> MeshletResolveLit {
  let r = rec_noextra(in.clip, in.row);
  if (!r.ok) { discard; }
  var o: MeshletResolveLit;
  o.depth = r.depth;
  hook_front = 1.0;
  analytic_uv = r.lit_uv;
  o.color = lit_fragment(r.v);
  return o;
}

fn rec_noreturns(pos: vec4<f32>, row: u32) -> MeshletRecord {
  var r: MeshletRecord;


  let pixel = u32(pos.y) * meshlet_screen.x + u32(pos.x);
  let slot = meshlet_attrs[pixel / 4u][pixel % 4u];
  r.ok = false;
  r.depth = 1.0;
  let at = meshlet_screen.z / 4u + (slot - 1u) * meshlet_screen.w;
  let a = meshlet_attrs[at];
  let b = meshlet_attrs[at + 1u];
  let c = meshlet_attrs[at + 2u];
  r.ok = true;
  r.depth = bitcast<f32>(a.w);
  r.v.clip = pos;
  r.v.world_pos = bitcast<vec3<f32>>(a.xyz);
  r.v.normal = meshlet_oct(unpack2x16snorm(b.x));
  r.v.uv = bitcast<vec2<f32>>(b.yz);
  r.v.color = unpack4x8unorm(b.w);

  r.v.local_pos = vec3<f32>(
    bitcast<f32>(meshlet_attrs[at + 4u].w),
    bitcast<f32>(meshlet_attrs[at + 6u].w),
    bitcast<f32>(meshlet_attrs[at + 7u].w),
  );
  r.v.tangent = vec4<f32>(meshlet_oct(unpack2x16snorm(c.x)), bitcast<f32>(c.y));
  if (r.v.tangent.w == 0.0) { r.v.tangent = vec4<f32>(0.0); }
  r.v.affine = vec3<f32>(r.v.uv, 1.0);
  r.v.look = bitcast<vec4<f32>>(meshlet_attrs[at + 3u]);
  r.prev_world = bitcast<vec3<f32>>(meshlet_attrs[at + 4u].xyz);

  analytic_uv = bitcast<vec4<f32>>(meshlet_attrs[at + 5u]);
  analytic_dx = bitcast<vec3<f32>>(meshlet_attrs[at + 6u].xyz);
  analytic_dy = bitcast<vec3<f32>>(meshlet_attrs[at + 7u].xyz);
  r.lit_uv = analytic_uv;
  if (meshlet_screen.w > 8u) {

    let lx = bitcast<vec4<f32>>(meshlet_attrs[at + 8u]);
    let ly = bitcast<vec4<f32>>(meshlet_attrs[at + 9u]);
    analytic_local_dx = lx.xyz;
    analytic_local_dy = ly.xyz;

    r.v.affine = vec3<f32>(lx.w, ly.w, 1.0);
    r.lit_uv = bitcast<vec4<f32>>(meshlet_attrs[at + 10u]);


    let d = meshlet_attrs[at + 11u];
    if (d.y != 0u && material.coat.w > 0.0) {
      r.v.normal = normalize(mix(normalize(r.v.normal), meshlet_oct(unpack2x16snorm(d.x)), clamp(material.coat.w, 0.0, 1.0)));
    }
  }
  r.object = c.w;
  return r;
}


@fragment
fn v_rec_noreturns(in: MeshletResolveOut) -> MeshletResolveLit {
  let r = rec_noreturns(in.clip, in.row);
  if (!r.ok) { discard; }
  var o: MeshletResolveLit;
  o.depth = r.depth;
  hook_front = 1.0;
  analytic_uv = r.lit_uv;
  o.color = lit_fragment(r.v);
  return o;
}

fn rec_noslotreturn(pos: vec4<f32>, row: u32) -> MeshletRecord {
  var r: MeshletRecord;


  let pixel = u32(pos.y) * meshlet_screen.x + u32(pos.x);
  let slot = meshlet_attrs[pixel / 4u][pixel % 4u];
  r.ok = false;
  r.depth = 1.0;
  let at = meshlet_screen.z / 4u + (slot - 1u) * meshlet_screen.w;
  let a = meshlet_attrs[at];
  let b = meshlet_attrs[at + 1u];
  let c = meshlet_attrs[at + 2u];
  if (c.z != row) { return r; }
  r.ok = true;
  r.depth = bitcast<f32>(a.w);
  r.v.clip = pos;
  r.v.world_pos = bitcast<vec3<f32>>(a.xyz);
  r.v.normal = meshlet_oct(unpack2x16snorm(b.x));
  r.v.uv = bitcast<vec2<f32>>(b.yz);
  r.v.color = unpack4x8unorm(b.w);

  r.v.local_pos = vec3<f32>(
    bitcast<f32>(meshlet_attrs[at + 4u].w),
    bitcast<f32>(meshlet_attrs[at + 6u].w),
    bitcast<f32>(meshlet_attrs[at + 7u].w),
  );
  r.v.tangent = vec4<f32>(meshlet_oct(unpack2x16snorm(c.x)), bitcast<f32>(c.y));
  if (r.v.tangent.w == 0.0) { r.v.tangent = vec4<f32>(0.0); }
  r.v.affine = vec3<f32>(r.v.uv, 1.0);
  r.v.look = bitcast<vec4<f32>>(meshlet_attrs[at + 3u]);
  r.prev_world = bitcast<vec3<f32>>(meshlet_attrs[at + 4u].xyz);

  analytic_uv = bitcast<vec4<f32>>(meshlet_attrs[at + 5u]);
  analytic_dx = bitcast<vec3<f32>>(meshlet_attrs[at + 6u].xyz);
  analytic_dy = bitcast<vec3<f32>>(meshlet_attrs[at + 7u].xyz);
  r.lit_uv = analytic_uv;
  if (meshlet_screen.w > 8u) {

    let lx = bitcast<vec4<f32>>(meshlet_attrs[at + 8u]);
    let ly = bitcast<vec4<f32>>(meshlet_attrs[at + 9u]);
    analytic_local_dx = lx.xyz;
    analytic_local_dy = ly.xyz;

    r.v.affine = vec3<f32>(lx.w, ly.w, 1.0);
    r.lit_uv = bitcast<vec4<f32>>(meshlet_attrs[at + 10u]);


    let d = meshlet_attrs[at + 11u];
    if (d.y != 0u && material.coat.w > 0.0) {
      r.v.normal = normalize(mix(normalize(r.v.normal), meshlet_oct(unpack2x16snorm(d.x)), clamp(material.coat.w, 0.0, 1.0)));
    }
  }
  r.object = c.w;
  return r;
}


@fragment
fn v_rec_noslotreturn(in: MeshletResolveOut) -> MeshletResolveLit {
  let r = rec_noslotreturn(in.clip, in.row);
  if (!r.ok) { discard; }
  var o: MeshletResolveLit;
  o.depth = r.depth;
  hook_front = 1.0;
  analytic_uv = r.lit_uv;
  o.color = lit_fragment(r.v);
  return o;
}

fn rec_norowreturn(pos: vec4<f32>, row: u32) -> MeshletRecord {
  var r: MeshletRecord;


  let pixel = u32(pos.y) * meshlet_screen.x + u32(pos.x);
  let slot = meshlet_attrs[pixel / 4u][pixel % 4u];
  r.ok = false;
  r.depth = 1.0;
  if (slot == 0u) { return r; }
  let at = meshlet_screen.z / 4u + (slot - 1u) * meshlet_screen.w;
  let a = meshlet_attrs[at];
  let b = meshlet_attrs[at + 1u];
  let c = meshlet_attrs[at + 2u];
  r.ok = true;
  r.depth = bitcast<f32>(a.w);
  r.v.clip = pos;
  r.v.world_pos = bitcast<vec3<f32>>(a.xyz);
  r.v.normal = meshlet_oct(unpack2x16snorm(b.x));
  r.v.uv = bitcast<vec2<f32>>(b.yz);
  r.v.color = unpack4x8unorm(b.w);

  r.v.local_pos = vec3<f32>(
    bitcast<f32>(meshlet_attrs[at + 4u].w),
    bitcast<f32>(meshlet_attrs[at + 6u].w),
    bitcast<f32>(meshlet_attrs[at + 7u].w),
  );
  r.v.tangent = vec4<f32>(meshlet_oct(unpack2x16snorm(c.x)), bitcast<f32>(c.y));
  if (r.v.tangent.w == 0.0) { r.v.tangent = vec4<f32>(0.0); }
  r.v.affine = vec3<f32>(r.v.uv, 1.0);
  r.v.look = bitcast<vec4<f32>>(meshlet_attrs[at + 3u]);
  r.prev_world = bitcast<vec3<f32>>(meshlet_attrs[at + 4u].xyz);

  analytic_uv = bitcast<vec4<f32>>(meshlet_attrs[at + 5u]);
  analytic_dx = bitcast<vec3<f32>>(meshlet_attrs[at + 6u].xyz);
  analytic_dy = bitcast<vec3<f32>>(meshlet_attrs[at + 7u].xyz);
  r.lit_uv = analytic_uv;
  if (meshlet_screen.w > 8u) {

    let lx = bitcast<vec4<f32>>(meshlet_attrs[at + 8u]);
    let ly = bitcast<vec4<f32>>(meshlet_attrs[at + 9u]);
    analytic_local_dx = lx.xyz;
    analytic_local_dy = ly.xyz;

    r.v.affine = vec3<f32>(lx.w, ly.w, 1.0);
    r.lit_uv = bitcast<vec4<f32>>(meshlet_attrs[at + 10u]);


    let d = meshlet_attrs[at + 11u];
    if (d.y != 0u && material.coat.w > 0.0) {
      r.v.normal = normalize(mix(normalize(r.v.normal), meshlet_oct(unpack2x16snorm(d.x)), clamp(material.coat.w, 0.0, 1.0)));
    }
  }
  r.object = c.w;
  return r;
}


@fragment
fn v_rec_norowreturn(in: MeshletResolveOut) -> MeshletResolveLit {
  let r = rec_norowreturn(in.clip, in.row);
  if (!r.ok) { discard; }
  var o: MeshletResolveLit;
  o.depth = r.depth;
  hook_front = 1.0;
  analytic_uv = r.lit_uv;
  o.color = lit_fragment(r.v);
  return o;
}

fn rec_noanalytic(pos: vec4<f32>, row: u32) -> MeshletRecord {
  var r: MeshletRecord;


  let pixel = u32(pos.y) * meshlet_screen.x + u32(pos.x);
  let slot = meshlet_attrs[pixel / 4u][pixel % 4u];
  r.ok = false;
  r.depth = 1.0;
  if (slot == 0u) { return r; }
  let at = meshlet_screen.z / 4u + (slot - 1u) * meshlet_screen.w;
  let a = meshlet_attrs[at];
  let b = meshlet_attrs[at + 1u];
  let c = meshlet_attrs[at + 2u];
  if (c.z != row) { return r; }
  r.ok = true;
  r.depth = bitcast<f32>(a.w);
  r.v.clip = pos;
  r.v.world_pos = bitcast<vec3<f32>>(a.xyz);
  r.v.normal = meshlet_oct(unpack2x16snorm(b.x));
  r.v.uv = bitcast<vec2<f32>>(b.yz);
  r.v.color = unpack4x8unorm(b.w);

  r.v.local_pos = vec3<f32>(
    bitcast<f32>(meshlet_attrs[at + 4u].w),
    bitcast<f32>(meshlet_attrs[at + 6u].w),
    bitcast<f32>(meshlet_attrs[at + 7u].w),
  );
  r.v.tangent = vec4<f32>(meshlet_oct(unpack2x16snorm(c.x)), bitcast<f32>(c.y));
  if (r.v.tangent.w == 0.0) { r.v.tangent = vec4<f32>(0.0); }
  r.v.affine = vec3<f32>(r.v.uv, 1.0);
  r.v.look = bitcast<vec4<f32>>(meshlet_attrs[at + 3u]);
  r.prev_world = bitcast<vec3<f32>>(meshlet_attrs[at + 4u].xyz);

  r.lit_uv = analytic_uv;
  if (meshlet_screen.w > 8u) {

    let lx = bitcast<vec4<f32>>(meshlet_attrs[at + 8u]);
    let ly = bitcast<vec4<f32>>(meshlet_attrs[at + 9u]);
    analytic_local_dx = lx.xyz;
    analytic_local_dy = ly.xyz;

    r.v.affine = vec3<f32>(lx.w, ly.w, 1.0);
    r.lit_uv = bitcast<vec4<f32>>(meshlet_attrs[at + 10u]);


    let d = meshlet_attrs[at + 11u];
    if (d.y != 0u && material.coat.w > 0.0) {
      r.v.normal = normalize(mix(normalize(r.v.normal), meshlet_oct(unpack2x16snorm(d.x)), clamp(material.coat.w, 0.0, 1.0)));
    }
  }
  r.object = c.w;
  return r;
}


@fragment
fn v_rec_noanalytic(in: MeshletResolveOut) -> MeshletResolveLit {
  let r = rec_noanalytic(in.clip, in.row);
  if (!r.ok) { discard; }
  var o: MeshletResolveLit;
  o.depth = r.depth;
  hook_front = 1.0;
  analytic_uv = r.lit_uv;
  o.color = lit_fragment(r.v);
  return o;
}

fn rec_onlyuv(pos: vec4<f32>, row: u32) -> MeshletRecord {
  var r: MeshletRecord;


  let pixel = u32(pos.y) * meshlet_screen.x + u32(pos.x);
  let slot = meshlet_attrs[pixel / 4u][pixel % 4u];
  r.ok = false;
  r.depth = 1.0;
  if (slot == 0u) { return r; }
  let at = meshlet_screen.z / 4u + (slot - 1u) * meshlet_screen.w;
  let a = meshlet_attrs[at];
  let b = meshlet_attrs[at + 1u];
  let c = meshlet_attrs[at + 2u];
  if (c.z != row) { return r; }
  r.ok = true;
  r.depth = bitcast<f32>(a.w);
  r.v.clip = pos;
  r.v.world_pos = bitcast<vec3<f32>>(a.xyz);
  r.v.normal = meshlet_oct(unpack2x16snorm(b.x));
  r.v.uv = bitcast<vec2<f32>>(b.yz);
  r.v.color = unpack4x8unorm(b.w);

  r.v.local_pos = vec3<f32>(
    bitcast<f32>(meshlet_attrs[at + 4u].w),
    bitcast<f32>(meshlet_attrs[at + 6u].w),
    bitcast<f32>(meshlet_attrs[at + 7u].w),
  );
  r.v.tangent = vec4<f32>(meshlet_oct(unpack2x16snorm(c.x)), bitcast<f32>(c.y));
  if (r.v.tangent.w == 0.0) { r.v.tangent = vec4<f32>(0.0); }
  r.v.affine = vec3<f32>(r.v.uv, 1.0);
  r.v.look = bitcast<vec4<f32>>(meshlet_attrs[at + 3u]);
  r.prev_world = bitcast<vec3<f32>>(meshlet_attrs[at + 4u].xyz);

  analytic_uv = bitcast<vec4<f32>>(meshlet_attrs[at + 5u]);
  r.lit_uv = analytic_uv;
  if (meshlet_screen.w > 8u) {

    let lx = bitcast<vec4<f32>>(meshlet_attrs[at + 8u]);
    let ly = bitcast<vec4<f32>>(meshlet_attrs[at + 9u]);
    analytic_local_dx = lx.xyz;
    analytic_local_dy = ly.xyz;

    r.v.affine = vec3<f32>(lx.w, ly.w, 1.0);
    r.lit_uv = bitcast<vec4<f32>>(meshlet_attrs[at + 10u]);


    let d = meshlet_attrs[at + 11u];
    if (d.y != 0u && material.coat.w > 0.0) {
      r.v.normal = normalize(mix(normalize(r.v.normal), meshlet_oct(unpack2x16snorm(d.x)), clamp(material.coat.w, 0.0, 1.0)));
    }
  }
  r.object = c.w;
  return r;
}


@fragment
fn v_rec_onlyuv(in: MeshletResolveOut) -> MeshletResolveLit {
  let r = rec_onlyuv(in.clip, in.row);
  if (!r.ok) { discard; }
  var o: MeshletResolveLit;
  o.depth = r.depth;
  hook_front = 1.0;
  analytic_uv = r.lit_uv;
  o.color = lit_fragment(r.v);
  return o;
}

fn rec_onlydx(pos: vec4<f32>, row: u32) -> MeshletRecord {
  var r: MeshletRecord;


  let pixel = u32(pos.y) * meshlet_screen.x + u32(pos.x);
  let slot = meshlet_attrs[pixel / 4u][pixel % 4u];
  r.ok = false;
  r.depth = 1.0;
  if (slot == 0u) { return r; }
  let at = meshlet_screen.z / 4u + (slot - 1u) * meshlet_screen.w;
  let a = meshlet_attrs[at];
  let b = meshlet_attrs[at + 1u];
  let c = meshlet_attrs[at + 2u];
  if (c.z != row) { return r; }
  r.ok = true;
  r.depth = bitcast<f32>(a.w);
  r.v.clip = pos;
  r.v.world_pos = bitcast<vec3<f32>>(a.xyz);
  r.v.normal = meshlet_oct(unpack2x16snorm(b.x));
  r.v.uv = bitcast<vec2<f32>>(b.yz);
  r.v.color = unpack4x8unorm(b.w);

  r.v.local_pos = vec3<f32>(
    bitcast<f32>(meshlet_attrs[at + 4u].w),
    bitcast<f32>(meshlet_attrs[at + 6u].w),
    bitcast<f32>(meshlet_attrs[at + 7u].w),
  );
  r.v.tangent = vec4<f32>(meshlet_oct(unpack2x16snorm(c.x)), bitcast<f32>(c.y));
  if (r.v.tangent.w == 0.0) { r.v.tangent = vec4<f32>(0.0); }
  r.v.affine = vec3<f32>(r.v.uv, 1.0);
  r.v.look = bitcast<vec4<f32>>(meshlet_attrs[at + 3u]);
  r.prev_world = bitcast<vec3<f32>>(meshlet_attrs[at + 4u].xyz);

  analytic_dx = bitcast<vec3<f32>>(meshlet_attrs[at + 6u].xyz);
  r.lit_uv = analytic_uv;
  if (meshlet_screen.w > 8u) {

    let lx = bitcast<vec4<f32>>(meshlet_attrs[at + 8u]);
    let ly = bitcast<vec4<f32>>(meshlet_attrs[at + 9u]);
    analytic_local_dx = lx.xyz;
    analytic_local_dy = ly.xyz;

    r.v.affine = vec3<f32>(lx.w, ly.w, 1.0);
    r.lit_uv = bitcast<vec4<f32>>(meshlet_attrs[at + 10u]);


    let d = meshlet_attrs[at + 11u];
    if (d.y != 0u && material.coat.w > 0.0) {
      r.v.normal = normalize(mix(normalize(r.v.normal), meshlet_oct(unpack2x16snorm(d.x)), clamp(material.coat.w, 0.0, 1.0)));
    }
  }
  r.object = c.w;
  return r;
}


@fragment
fn v_rec_onlydx(in: MeshletResolveOut) -> MeshletResolveLit {
  let r = rec_onlydx(in.clip, in.row);
  if (!r.ok) { discard; }
  var o: MeshletResolveLit;
  o.depth = r.depth;
  hook_front = 1.0;
  analytic_uv = r.lit_uv;
  o.color = lit_fragment(r.v);
  return o;
}

fn rec_onlydy(pos: vec4<f32>, row: u32) -> MeshletRecord {
  var r: MeshletRecord;


  let pixel = u32(pos.y) * meshlet_screen.x + u32(pos.x);
  let slot = meshlet_attrs[pixel / 4u][pixel % 4u];
  r.ok = false;
  r.depth = 1.0;
  if (slot == 0u) { return r; }
  let at = meshlet_screen.z / 4u + (slot - 1u) * meshlet_screen.w;
  let a = meshlet_attrs[at];
  let b = meshlet_attrs[at + 1u];
  let c = meshlet_attrs[at + 2u];
  if (c.z != row) { return r; }
  r.ok = true;
  r.depth = bitcast<f32>(a.w);
  r.v.clip = pos;
  r.v.world_pos = bitcast<vec3<f32>>(a.xyz);
  r.v.normal = meshlet_oct(unpack2x16snorm(b.x));
  r.v.uv = bitcast<vec2<f32>>(b.yz);
  r.v.color = unpack4x8unorm(b.w);

  r.v.local_pos = vec3<f32>(
    bitcast<f32>(meshlet_attrs[at + 4u].w),
    bitcast<f32>(meshlet_attrs[at + 6u].w),
    bitcast<f32>(meshlet_attrs[at + 7u].w),
  );
  r.v.tangent = vec4<f32>(meshlet_oct(unpack2x16snorm(c.x)), bitcast<f32>(c.y));
  if (r.v.tangent.w == 0.0) { r.v.tangent = vec4<f32>(0.0); }
  r.v.affine = vec3<f32>(r.v.uv, 1.0);
  r.v.look = bitcast<vec4<f32>>(meshlet_attrs[at + 3u]);
  r.prev_world = bitcast<vec3<f32>>(meshlet_attrs[at + 4u].xyz);

  analytic_dy = bitcast<vec3<f32>>(meshlet_attrs[at + 7u].xyz);
  r.lit_uv = analytic_uv;
  if (meshlet_screen.w > 8u) {

    let lx = bitcast<vec4<f32>>(meshlet_attrs[at + 8u]);
    let ly = bitcast<vec4<f32>>(meshlet_attrs[at + 9u]);
    analytic_local_dx = lx.xyz;
    analytic_local_dy = ly.xyz;

    r.v.affine = vec3<f32>(lx.w, ly.w, 1.0);
    r.lit_uv = bitcast<vec4<f32>>(meshlet_attrs[at + 10u]);


    let d = meshlet_attrs[at + 11u];
    if (d.y != 0u && material.coat.w > 0.0) {
      r.v.normal = normalize(mix(normalize(r.v.normal), meshlet_oct(unpack2x16snorm(d.x)), clamp(material.coat.w, 0.0, 1.0)));
    }
  }
  r.object = c.w;
  return r;
}


@fragment
fn v_rec_onlydy(in: MeshletResolveOut) -> MeshletResolveLit {
  let r = rec_onlydy(in.clip, in.row);
  if (!r.ok) { discard; }
  var o: MeshletResolveLit;
  o.depth = r.depth;
  hook_front = 1.0;
  analytic_uv = r.lit_uv;
  o.color = lit_fragment(r.v);
  return o;
}

fn rec_nouv(pos: vec4<f32>, row: u32) -> MeshletRecord {
  var r: MeshletRecord;


  let pixel = u32(pos.y) * meshlet_screen.x + u32(pos.x);
  let slot = meshlet_attrs[pixel / 4u][pixel % 4u];
  r.ok = false;
  r.depth = 1.0;
  if (slot == 0u) { return r; }
  let at = meshlet_screen.z / 4u + (slot - 1u) * meshlet_screen.w;
  let a = meshlet_attrs[at];
  let b = meshlet_attrs[at + 1u];
  let c = meshlet_attrs[at + 2u];
  if (c.z != row) { return r; }
  r.ok = true;
  r.depth = bitcast<f32>(a.w);
  r.v.clip = pos;
  r.v.world_pos = bitcast<vec3<f32>>(a.xyz);
  r.v.normal = meshlet_oct(unpack2x16snorm(b.x));
  r.v.uv = bitcast<vec2<f32>>(b.yz);
  r.v.color = unpack4x8unorm(b.w);

  r.v.local_pos = vec3<f32>(
    bitcast<f32>(meshlet_attrs[at + 4u].w),
    bitcast<f32>(meshlet_attrs[at + 6u].w),
    bitcast<f32>(meshlet_attrs[at + 7u].w),
  );
  r.v.tangent = vec4<f32>(meshlet_oct(unpack2x16snorm(c.x)), bitcast<f32>(c.y));
  if (r.v.tangent.w == 0.0) { r.v.tangent = vec4<f32>(0.0); }
  r.v.affine = vec3<f32>(r.v.uv, 1.0);
  r.v.look = bitcast<vec4<f32>>(meshlet_attrs[at + 3u]);
  r.prev_world = bitcast<vec3<f32>>(meshlet_attrs[at + 4u].xyz);

  analytic_dx = bitcast<vec3<f32>>(meshlet_attrs[at + 6u].xyz);
  analytic_dy = bitcast<vec3<f32>>(meshlet_attrs[at + 7u].xyz);
  r.lit_uv = analytic_uv;
  if (meshlet_screen.w > 8u) {

    let lx = bitcast<vec4<f32>>(meshlet_attrs[at + 8u]);
    let ly = bitcast<vec4<f32>>(meshlet_attrs[at + 9u]);
    analytic_local_dx = lx.xyz;
    analytic_local_dy = ly.xyz;

    r.v.affine = vec3<f32>(lx.w, ly.w, 1.0);
    r.lit_uv = bitcast<vec4<f32>>(meshlet_attrs[at + 10u]);


    let d = meshlet_attrs[at + 11u];
    if (d.y != 0u && material.coat.w > 0.0) {
      r.v.normal = normalize(mix(normalize(r.v.normal), meshlet_oct(unpack2x16snorm(d.x)), clamp(material.coat.w, 0.0, 1.0)));
    }
  }
  r.object = c.w;
  return r;
}


@fragment
fn v_rec_nouv(in: MeshletResolveOut) -> MeshletResolveLit {
  let r = rec_nouv(in.clip, in.row);
  if (!r.ok) { discard; }
  var o: MeshletResolveLit;
  o.depth = r.depth;
  hook_front = 1.0;
  analytic_uv = r.lit_uv;
  o.color = lit_fragment(r.v);
  return o;
}

fn rec_notangentif(pos: vec4<f32>, row: u32) -> MeshletRecord {
  var r: MeshletRecord;


  let pixel = u32(pos.y) * meshlet_screen.x + u32(pos.x);
  let slot = meshlet_attrs[pixel / 4u][pixel % 4u];
  r.ok = false;
  r.depth = 1.0;
  if (slot == 0u) { return r; }
  let at = meshlet_screen.z / 4u + (slot - 1u) * meshlet_screen.w;
  let a = meshlet_attrs[at];
  let b = meshlet_attrs[at + 1u];
  let c = meshlet_attrs[at + 2u];
  if (c.z != row) { return r; }
  r.ok = true;
  r.depth = bitcast<f32>(a.w);
  r.v.clip = pos;
  r.v.world_pos = bitcast<vec3<f32>>(a.xyz);
  r.v.normal = meshlet_oct(unpack2x16snorm(b.x));
  r.v.uv = bitcast<vec2<f32>>(b.yz);
  r.v.color = unpack4x8unorm(b.w);

  r.v.local_pos = vec3<f32>(
    bitcast<f32>(meshlet_attrs[at + 4u].w),
    bitcast<f32>(meshlet_attrs[at + 6u].w),
    bitcast<f32>(meshlet_attrs[at + 7u].w),
  );
  r.v.tangent = vec4<f32>(meshlet_oct(unpack2x16snorm(c.x)), bitcast<f32>(c.y));
  r.v.affine = vec3<f32>(r.v.uv, 1.0);
  r.v.look = bitcast<vec4<f32>>(meshlet_attrs[at + 3u]);
  r.prev_world = bitcast<vec3<f32>>(meshlet_attrs[at + 4u].xyz);

  analytic_uv = bitcast<vec4<f32>>(meshlet_attrs[at + 5u]);
  analytic_dx = bitcast<vec3<f32>>(meshlet_attrs[at + 6u].xyz);
  analytic_dy = bitcast<vec3<f32>>(meshlet_attrs[at + 7u].xyz);
  r.lit_uv = analytic_uv;
  if (meshlet_screen.w > 8u) {

    let lx = bitcast<vec4<f32>>(meshlet_attrs[at + 8u]);
    let ly = bitcast<vec4<f32>>(meshlet_attrs[at + 9u]);
    analytic_local_dx = lx.xyz;
    analytic_local_dy = ly.xyz;

    r.v.affine = vec3<f32>(lx.w, ly.w, 1.0);
    r.lit_uv = bitcast<vec4<f32>>(meshlet_attrs[at + 10u]);


    let d = meshlet_attrs[at + 11u];
    if (d.y != 0u && material.coat.w > 0.0) {
      r.v.normal = normalize(mix(normalize(r.v.normal), meshlet_oct(unpack2x16snorm(d.x)), clamp(material.coat.w, 0.0, 1.0)));
    }
  }
  r.object = c.w;
  return r;
}


@fragment
fn v_rec_notangentif(in: MeshletResolveOut) -> MeshletResolveLit {
  let r = rec_notangentif(in.clip, in.row);
  if (!r.ok) { discard; }
  var o: MeshletResolveLit;
  o.depth = r.depth;
  hook_front = 1.0;
  analytic_uv = r.lit_uv;
  o.color = lit_fragment(r.v);
  return o;
}

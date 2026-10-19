// The terrain's normal map: one fullscreen triangle over the height
// field's grid, central differences of the heights (one-sided at the
// edges), written as the normal's x and z (y is up and positive; the
// lit pass rebuilds it) into an Rg16Float target, mipmapped afterwards.

struct NormalsParams {
  extent: vec4<f32>,  // xy: size along x and z (m), zw: samples along x and z
};
@group(0) @binding(0) var<uniform> params: NormalsParams;
@group(0) @binding(1) var heights: texture_2d<f32>;

@vertex
fn vs(@builtin(vertex_index) i: u32) -> @builtin(position) vec4<f32> {
  let x = f32(i32(i & 1u) * 4 - 1);
  let y = f32(i32(i & 2u) * 2 - 1);
  return vec4<f32>(x, y, 0.0, 1.0);
}

@fragment
fn fs(@builtin(position) pos: vec4<f32>) -> @location(0) vec4<f32> {
  let n = vec2<i32>(params.extent.zw);
  let cell = params.extent.xy / max(params.extent.zw - 1.0, vec2<f32>(1.0));
  let c = vec2<i32>(pos.xy);
  let x0 = max(c.x - 1, 0);
  let x1 = min(c.x + 1, n.x - 1);
  let z0 = max(c.y - 1, 0);
  let z1 = min(c.y + 1, n.y - 1);
  let hx = (textureLoad(heights, vec2<i32>(x1, c.y), 0).r - textureLoad(heights, vec2<i32>(x0, c.y), 0).r) / (f32(max(x1 - x0, 1)) * cell.x);
  let hz = (textureLoad(heights, vec2<i32>(c.x, z1), 0).r - textureLoad(heights, vec2<i32>(c.x, z0), 0).r) / (f32(max(z1 - z0, 1)) * cell.y);
  let normal = normalize(vec3<f32>(-hx, 1.0, -hz));
  return vec4<f32>(normal.x, normal.z, 0.0, 1.0);
}

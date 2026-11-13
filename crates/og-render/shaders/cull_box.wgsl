// An instance's box against the depth pyramid (cull_hiz.wgsl), shared by
// the instances' cull (cull.wgsl) and the crowds' (crowd_cull.wgsl). The
// including module defines `params` with `sizes` (y: the pyramid's levels,
// zw: the screen's size) and `view_proj`.

// An instance's box (its local bounds through its transform) on screen.
fn screen_box(m: mat4x4<f32>, center: vec3<f32>, extent: vec3<f32>) -> ScreenBox {
  let size = vec2<f32>(params.sizes.zw);
  var box = hiz_nothing();
  for (var i = 0u; i < 8u && box.ok; i++) {
    let corner = center + extent * hiz_corner(i);
    hiz_extend(&box, mul_point(params.view_proj, m * vec4<f32>(corner, 1.0)), size);
  }
  return box;
}

// Whether the instance's box lies behind the occluders' depth everywhere
// it covers.
fn occluded(m: mat4x4<f32>, center: vec3<f32>, extent: vec3<f32>) -> bool {
  return hiz_hidden(screen_box(m, center, extent), params.sizes.zw, params.sizes.y);
}

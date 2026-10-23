// One face of a light cookie packed into a local shadow array layer
// (area.rs): the texture's linear colour, each channel rounded to 8 bits,
// as the exact integer r * 65536 + g * 256 + b that area.wgsl unpacks.
// Prepended with fullscreen.wgsl.

struct CookieFace { face: f32, faces: f32, pad: vec2<f32> };
@group(0) @binding(0) var<uniform> cookie_face: CookieFace;
@group(0) @binding(1) var cookie_texture: texture_2d<f32>;
@group(0) @binding(2) var cookie_sampler: sampler;

@fragment
fn fs_cookie(in: VsOut) -> @location(0) vec4<f32> {
  // The faces are stacked top to bottom; a face's filter stays inside it.
  let h = f32(textureDimensions(cookie_texture).y);
  let face_h = h / cookie_face.faces;
  let top = cookie_face.face * face_h;
  let y = clamp(top + in.uv.y * face_h, top + 0.5, top + face_h - 0.5);
  let uv = vec2<f32>(in.uv.x, y / h);
  let c = clamp(textureSampleLevel(cookie_texture, cookie_sampler, uv, 0.0).rgb, vec3<f32>(0.0), vec3<f32>(1.0));
  let b = floor(c * 255.0 + 0.5);
  return vec4<f32>(b.r * 65536.0 + b.g * 256.0 + b.b, 0.0, 0.0, 1.0);
}

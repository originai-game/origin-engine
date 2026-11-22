// A packed-alpha video frame (`alpha true`) into its stream texture, on
// the web: the browser copied the decoded picture, twice the frame's
// height, into `packed` (8-bit, sRGB-encoded values as stored); its upper
// half is the colour, its lower half the matte as grey, which the browser
// already brought from limited to full range. The target is the stream
// texture (sRGB), so the colour is decoded here and encoded by the target
// unchanged; the matte is the alpha as it is.

@group(0) @binding(0) var packed: texture_2d<f32>;

@vertex
fn vs(@builtin(vertex_index) i: u32) -> @builtin(position) vec4<f32> {
  return fullscreen_clip(i);
}

@fragment
fn fs(@builtin(position) p: vec4<f32>) -> @location(0) vec4<f32> {
  let half = i32(textureDimensions(packed).y / 2u);
  let at = vec2<i32>(p.xy);
  let colour = textureLoad(packed, at, 0);
  let matte = textureLoad(packed, at + vec2<i32>(0, half), 0);
  return vec4<f32>(srgb_decode(colour.rgb), matte.r);
}

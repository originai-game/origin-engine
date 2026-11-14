// The effect passes' uniform (`FxPostUniform`, post.rs): shared by the
// fullscreen effects of fxpost.wgsl and the contact-shadow compute pass
// (contact.wgsl).
struct FxPost {
  proj: mat4x4<f32>,
  inv_proj: mat4x4<f32>,
  view: mat4x4<f32>,          // world -> view (normals)
  screen: vec4<f32>,          // full-resolution w, h, 1/w, 1/h
  cam: vec4<f32>,             // x: far, y: 1 orthographic, z: near, w: the AO debug views (1: fract(depth), DebugView::AoDepth; 2: AO unblurred, DebugView::AoRaw)
  ao: vec4<f32>,              // x: radius (world), y: intensity, z: the AO targets' divisor (1, or 2 at half resolution)
  ao2: vec4<f32>,             // x: radius per metre of depth (0: fixed), y: 1 screen method, z: the scale GTAO's raw term is stored over (fs_gtao)
  outline_color: vec4<f32>,   // the scene's outline (linear rgb, coverage)
  outline: vec4<f32>,         // x: scene width (0: none), y: depth threshold, z: normal threshold
  dof: vec4<f32>,             // x: focus distance, y: focus range, z: max blur (full-res px), w: falloff
  dof2: vec4<f32>,            // x: taps
  mb: vec4<f32>,              // x: shutter, y: samples, z: max px
  god: vec4<f32>,             // xy: sun uv, z: decay, w: density
  god2: vec4<f32>,            // x: samples, y: 1 when the sun is in front, z: intensity, w: sky threshold
  god_tint: vec4<f32>,
  bands: vec4<f32>,           // x: depth bands (0: none), y: near, z: far, w: flatten
  bands_near: vec4<f32>,      // the nearest band's colour (linear)
  bands_far: vec4<f32>,       // the farthest band's colour (linear)
  blur: vec4<f32>,            // the screen blur: x kind (0 off, 1 zoom, 2 spin, 3 directional), y strength, zw centre (uv)
  blur2: vec4<f32>,           // x: taps, yz: directional's offset (uv) at t = 1
  gi0: vec4<f32>,             // screen-space GI (ssgi.wgsl): x max distance, y thickness, z intensity (0: off), w steps
  gi1: vec4<f32>,             // x: rays, y: the noise's frame offset (px), z: 1 with last frame's lit image (tex_a), w: the occlusion from 0 the rays, 1 GTAO, 2 SSAO
  gi2: vec4<f32>,             // x: how much the rays' hits occlude the ambient light
  // Fog of war (post/sight.rs): row 0 the map's min corner in its plane
  // (from the frame's origin) and 1 over its extent, row 1 the unseen's
  // colour and how much of the scene shows where it is only explored, row
  // 2 x: 1 drawn, y: 1 the x, y plane (0: the ground, x and z).
  sight: array<vec4<f32>, 3>,
  // Screen-space contact shadows (contact.wgsl): rows 0 to 2 the contact
  // lights' view-space positions (w the frame's tag in the masks' keys,
  // 0: not drawn), row 3 Bend's x surface
  // thickness, y bilinear threshold, z shadow contrast, w sample count.
  contact: array<vec4<f32>, 4>,
};

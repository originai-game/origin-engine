import { file, ref } from '@origingame/engine/author';

// Shelf, saddle, headland and seabed from tools/terrain.ts; the sea is y = 0.
export default file.terrain({
  source: { kind: 'image', source: 'assets/terrain/coast-height.png', height: [-80, 80] },
  size: [480, 480],
  control: ['assets/terrain/coast-control.png'],
  heightBlend: 0.5,
  layers: [
    // Control red: turf.
    {
      albedo: ref('base/terrain/textures/aerial_grass_rock_diff_1k'),
      normal: ref('base/terrain/textures/aerial_grass_rock_nor_gl_1k'),
      color: [0.62, 0.74, 0.5], roughness: 0.95, tile: 7,
      antiTiling: 0.7, macroVariation: 0.6,
    },
    // Green: cliff rock projected over steep faces.
    {
      albedo: ref('base/terrain/textures/rock_face_03_grey_1k'),
      normal: ref('base/terrain/textures/rock_face_03_nor_gl_1k'),
      color: [0.78, 0.78, 0.8], roughness: 0.9, tile: 14,
      antiTiling: 0.5, normalStrength: 1.6, triplanar: true, macroVariation: 0.5,
    },
    // Blue: the trodden path.
    {
      albedo: ref('base/terrain/textures/forest_ground_04_diff_1k'),
      normal: ref('base/terrain/textures/forest_ground_04_nor_gl_1k'),
      color: [0.95, 0.8, 0.66], roughness: 1, tile: 2.5,
      antiTiling: 0.6, macroVariation: 0.3,
    },
    // Alpha: wave-washed rock and seabed.
    {
      albedo: ref('base/terrain/textures/rock_face_03_grey_1k'),
      normal: ref('base/terrain/textures/rock_face_03_nor_gl_1k'),
      color: [0.3, 0.31, 0.33], roughness: 0.45, tile: 14, triplanar: true,
    },
  ],
});

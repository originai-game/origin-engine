import { file, ref } from "@origingame/engine/author";

export default file.look({
  // Broad anisotropic brush strokes (the widest the filter takes) that flatten
  // the lit scene into colour masses, each mass laid in visible strokes with
  // ragged edges, pigment pooling at their rims and bleed between them, a few
  // soft tone steps, all on a woven canvas (the engine's @canvas/linen).
  name: "Painterly 3D",
  mood: "warm",
  ui: "@themes/fantasy",
  Stylise: {
    paintRadius: 8,
    paintAnisotropy: 0.9,
    paintEdgeDarkening: 0.7,
    paintBleed: 0.5,
    paintStrokes: 0.7,
    paintCanvas: ref("@canvas/linen.png"),
    paintCanvasStrength: 0.7,
    posterize: 12,
    posterizeMode: "tone",
  },
  PostEffects: {
    vignette: true,
    vignetteIntensity: 0.25,
  },
  // The art contract (docs/art.md §2.1), from the catalogue entry's palette and
  // assets: a free palette graded by a LUT; 10k to 30k triangles; hand-painted
  // at up to 1 texel per screen pixel on 250 px characters. Props are held to
  // the same 30k, and no texture is longer than 2048 texels (1 texel per
  // screen pixel on a model filling a 1080-line frame).
  // The palette is free (the catalogue's), so it states no colours: measure
  // reports the palette facts absent, never against invented colours.
  ArtTexels: {
    density: 128,
    maxSize: 2048,
  },
  ArtShape: {
    propFaces: 30000,
    characterFaces: 30000,
  },
});

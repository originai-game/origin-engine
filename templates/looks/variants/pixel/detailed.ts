import { file } from "@origingame/engine/author";

export default file.look({
  // A tall grid with twelve-tone ramps and a fine dither.
  // The detailed variant of pixel (`variant "detailed" `): only what it changes of
  // the preset, a field it turns off written at its default.
  name: "Densely detailed pixel art",
  mood: "warm",
  Stylise: {
    pixelHeight: 270,
    posterize: 12,
    dither: "bayer8",
    ditherStrength: 0.6,
  },
  ArtPalette: {
    maxColors: 128,
    rampSteps: 0,
  },
  ArtPixels: {
    unit: 32,
    logicalHeight: 270,
    spriteSizes: [32, 48, 64],
  },
  ArtLine: {
    width: 0,
    place: "outside",
  },
});

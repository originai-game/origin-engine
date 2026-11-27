import { file } from "@origingame/engine/author";

export default file.look({
  // A finer grid, eight tones per channel through a gentle Bayer 4 dither: the
  // frame's quantise holds anything to the look, while art the cook conformed
  // (64 colours in 6-step ramps, ArtPalette) keeps its own colours.
  name: "Pixel art",
  mood: "bright",
  ui: "@themes/pixel",
  // The UI's seed colours (UiSeeds), so the default style matches the
  // scene: a sixteen-colour blue and gold over navy.
  uiSeeds: {
    primary: "#3b5dc9",
    accent: "#ffcd75",
    neutral: "#333c57",
  },
  antialias: "off",
  // 270 lines: a whole 4x at 1080p, 8x at 2160p, so those outputs fill the
  // frame without a border (224 left one at 1080p).
  Stylise: {
    pixelHeight: 270,
    pixelSnap: true,
    posterize: 8,
    dither: "bayer4",
    ditherStrength: 0.5,
  },
  RenderSettings: {
    toneMapping: "none",
  },
  // The art contract (docs/art.md §2.1), from the catalogue entry's palette and
  // assets: 32 to 64 colours in 4- to 6-step ramps; 32 to 48 px characters on 16
  // x 16 tiles at the preset's 270 lines; a 1-pixel dark line inside the sprite.
  ArtPalette: {
    maxColors: 64,
    rampSteps: 6,
  },
  ArtPixels: {
    unit: 16,
    logicalHeight: 270,
    spriteSizes: [16, 32, 48],
    rotation: "cleanEdge",
    snap: true,
  },
  ArtLine: {
    width: 1,
    place: "inside",
  },
  ArtCamera: {
    projection: "orthographic",
    pixelPerfect: true,
  },
});

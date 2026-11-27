import { file } from "@origingame/engine/author";

export default file.look({
  // Flat cel shading with thick outlines, no shadows to speak of: the cel
  // bands are the toon material's, the flat fills the art's. No Stylise
  // posterize: it rounds each channel on its own, so a sepia print's close
  // channels land on different steps and turn olive and pink (at 4 tones,
  // still visibly at 8; tests/evidence/art/samples/NOTES.md).
  // Film grain two pixels across, as the stock of an early theatrical cartoon:
  // it shows on the flat fills and leaves the ink alone.
  // The rubber-hose variant of cartoon-2d (`variant "rubber-hose" `): only what it changes of
  // the preset, a field it turns off written at its default.
  name: "Rubber-hose animation",
  mood: "high-contrast",
  Stylise: {
    posterize: 0,
    posterizeMode: "channel",
  },
  PostEffects: {
    vignette: false,
    vignetteIntensity: 0.35,
    filmGrain: 0.08,
    filmGrainSize: 2,
  },
  ArtPalette: {
    maxColors: 24,
    rampSteps: 0,
  },
  ArtLine: {
    width: 3,
    color: "#101010",
  },
  MaterialStyle: {
    toon: true,
    outlineWidth: 3,
    outlineColor: "#101010",
    shadowTint: "#ffffff",
    shadowTintAmount: 1,
  },
  RenderSettings: {
    toneMapping: "none",
  },
  ArtMotion: {
    stepFps: 12,
    squash: 1,
  },
});

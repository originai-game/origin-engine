import { file } from "@origingame/engine/author";

export default file.look({
  // Soft rim light, a touch of grain and painterly smoothing: handmade surfaces (set AnimationPlayer.stepFps 12 on the characters for stop motion on twos).
  // The clay variant of toy-3d (`variant "clay" `): only what it changes of
  // the preset, a field it turns off written at its default.
  name: "Sculpted clay & stop-motion",
  mood: "warm",
  MaterialStyle: {
    rimIntensity: 0.1,
    rimPower: 2.5,
    rimColor: "#fff2e0",
    surface: "clay",
  },
  RenderSettings: {
    toneMapping: "aces",
  },
  PostEffects: {
    filmGrain: 0.03,
    vignette: false,
    vignetteIntensity: 0.35,
  },
  ArtPalette: {
    maxColors: 0,
  },
  ArtTexels: {
    maxSize: 0,
  },
  ArtShape: {
    propFaces: 5000,
    characterFaces: 15000,
  },
  Stylise: {
    paintRadius: 2,
    paintAnisotropy: 0.2,
  },
  ArtMotion: {
    stepFps: 12,
  },
});

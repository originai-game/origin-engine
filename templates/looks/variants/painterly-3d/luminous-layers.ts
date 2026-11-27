import { file } from "@origingame/engine/author";

export default file.look({
  // Five depth bands tinted from a dark foreground to a glowing distance, with bloom.
  // The luminous-layers variant of painterly-3d (`variant "luminous-layers" `): only what it changes of
  // the preset, a field it turns off written at its default.
  name: "Luminous layered environments",
  mood: "cold",
  Stylise: {
    depthBands: 5,
    depthBandsFit: true,
    depthBandsNearColor: "#1a1430",
    depthBandsFarColor: "#ffd9a0",
    depthBandsFlatten: 0.6,
    paintRadius: 0,
    paintAnisotropy: 0,
    paintEdgeDarkening: 0,
    paintBleed: 0,
    paintCanvasStrength: 0,
    posterize: 0,
    posterizeMode: "channel",
  },
  PostEffects: {
    vignette: false,
    vignetteIntensity: 0.35,
  },
  ArtTexels: {
    density: 0,
  },
  ArtShape: {
    propFaces: 10000,
    characterFaces: 0,
  },
  RenderSettings: {
    bloom: true,
    bloomIntensity: 0.5,
  },
});

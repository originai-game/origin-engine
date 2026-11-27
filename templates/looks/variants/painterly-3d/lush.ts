import { file } from "@origingame/engine/author";

export default file.look({
  // Smaller, lighter brush strokes over a bright, blooming scene.
  // The lush variant of painterly-3d (`variant "lush" `): only what it changes of
  // the preset, a field it turns off written at its default.
  name: "Lush painted worlds",
  Stylise: {
    paintRadius: 5,
    paintAnisotropy: 0.5,
    paintEdgeDarkening: 0,
    paintBleed: 0,
    paintStrokes: 0.5,
    paintCanvasStrength: 0,
    posterize: 0,
    posterizeMode: "channel",
  },
  PostEffects: {
    vignette: false,
    vignetteIntensity: 0.35,
  },
  ArtShape: {
    propFaces: 8000,
    characterFaces: 0,
  },
  RenderSettings: {
    bloom: true,
    bloomIntensity: 0.4,
    exposure: 1.05,
  },
});

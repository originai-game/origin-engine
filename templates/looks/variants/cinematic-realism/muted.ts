import { file } from "@origingame/engine/author";

export default file.look({
  // AgX at lower exposure, grey fog and a strong vignette: a lit stage.
  // The muted variant of cinematic-realism (`variant "muted" `): only what it changes of
  // the preset, a field it turns off written at its default.
  name: "Muted theatrical lighting",
  mood: "dark",
  RenderSettings: {
    exposure: 0.8,
    bloom: false,
    bloomIntensity: 0.6,
    autoExposure: false,
    adaptSpeedUp: 0,
    adaptSpeedDown: 0,
  },
  PostEffects: {
    vignetteIntensity: 0.5,
    chromaticAberration: 0,
    filmGrain: 0,
  },
  ArtTexels: {
    density: 0,
    filter: "any",
    mips: "any",
  },
  ArtShape: {
    propFaces: 10000,
    characterFaces: 10000,
  },
  ArtCamera: {
    projection: "any",
  },
  Fog: {
    enabled: true,
    color: "#6a6a70",
    density: 0.02,
  },
});

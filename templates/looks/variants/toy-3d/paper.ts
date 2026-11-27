import { file, ref } from "@origingame/engine/author";

export default file.look({
  // Flat facets with a fine contour, ambient occlusion in the folds, the engine's paper surface on every material and a paper grain over all of it (the engine's @canvas/paper). No posterize: its per-channel rounding turns near-neutral card colours another hue.
  // The paper variant of toy-3d (`variant "paper" `): only what it changes of
  // the preset, a field it turns off written at its default.
  name: "Folded paper & papercraft",
  mood: "soft",
  MaterialStyle: {
    flatShading: true,
    outlineWidth: 1,
    outlineColor: "#5a4a3a",
    surface: "paper",
  },
  RenderSettings: {
    toneMapping: "aces",
  },
  PostEffects: {
    vignette: false,
    vignetteIntensity: 0.35,
  },
  ArtPalette: {
    maxColors: 16,
  },
  ArtTexels: {
    maxSize: 0,
  },
  ArtShape: {
    propFaces: 2000,
    characterFaces: 600,
    normals: "flat",
    bevel: 0,
  },
  Stylise: {
    paintCanvas: ref("@canvas/paper.png"),
    paintCanvasStrength: 0.5,
  },
});

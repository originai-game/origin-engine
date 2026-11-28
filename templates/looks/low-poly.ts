import { file } from "@origingame/engine/author";

export default file.look({
  // Broad planar facets with toy-bright colours and tactile contacts.
  // Tiny bevels catch light; unlike the former toy preset the playfield
  // stays sharp and the low polygon budget is not smoothed away.
  name: "Low-poly",
  mood: "bright",
  ui: "@themes/candy",
  // The UI's seed colours (UiSeeds), so the default style matches the
  // scene: a soft teal over warm stone.
  uiSeeds: {
    primary: "#2f9c8a",
    neutral: "#8a8577",
  },
  MaterialStyle: {
    flatShading: true,
    surface: "plastic",
  },
  RenderSettings: {
    toneMapping: "agx",
    autoExposure: true,
    adaptSpeedUp: 1,
    adaptSpeedDown: 3,
  },
  PostEffects: {
    vignette: true,
    vignetteIntensity: 0.2,
    ao: true,
    aoIntensity: 1.2,
  },
  // The art contract (docs/art.md §2.1), from the catalogue entry's palette and
  // assets: 16 to 32 colours, one per material; characters 300 to 1500
  // triangles, props 50 to 500, flat facets; no textures: with flat normals
  // and no ArtTexels the cook bakes each face one colour (the texture's mean
  // over it, cut to the palette), so the facets read as colour fields.
  ArtPalette: {
    maxColors: 32,
  },
  ArtShape: {
    propFaces: 500,
    characterFaces: 1500,
    normals: "flat",
    bevel: 0.015,
  },
});

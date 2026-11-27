import { file } from "@origingame/engine/author";

export default file.look({
  // AgX, a little bloom, grain, aberration, a vignette and eye-like exposure adaptation.
  name: "Cinematic realism",
  mood: "cold",
  ui: "@themes/minimal",
  // The UI's seed colours (UiSeeds), so the default style matches the
  // scene: a graded steel blue over cold greys.
  uiSeeds: {
    primary: "#5b7a99",
    neutral: "#56606b",
  },
  RenderSettings: {
    toneMapping: "agx",
    bloom: true,
    bloomIntensity: 0.3,
    autoExposure: true,
    adaptSpeedUp: 1,
    adaptSpeedDown: 3,
  },
  PostEffects: {
    vignette: true,
    vignetteIntensity: 0.3,
    chromaticAberration: 0.5,
    filmGrain: 0.03,
  },
  // The art contract (docs/art.md §2.1), from the catalogue entry's palette and
  // assets: a free graded palette; up to 100k triangles per hero; PBR textures
  // at up to 2 texels per screen pixel on 400 px characters; real lenses. A
  // prop is held to the hero's 100k, and no texture is longer than 4096 texels
  // (2 texels per screen pixel on a model filling a 1080-line frame).
  // The palette is free (the catalogue's), so it states no colours: measure
  // reports the palette facts absent, never against invented colours.
  ArtTexels: {
    density: 512,
    maxSize: 4096,
    filter: "linear",
    mips: "full",
  },
  ArtShape: {
    propFaces: 100000,
    characterFaces: 100000,
  },
  ArtCamera: {
    projection: "perspective",
  },
});

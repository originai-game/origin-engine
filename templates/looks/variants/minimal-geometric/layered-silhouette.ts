import { file } from "@origingame/engine/author";

export default file.look({
  // Six flat depth bands from a dark foreground to a pale distance.
  // The layered-silhouette variant of minimal-geometric (`variant "layered-silhouette" `): only what it changes of
  // the preset, a field it turns off written at its default.
  name: "Layered silhouettes",
  mood: "high-contrast",
  MaterialStyle: {
    toon: false,
    toonSoftness: 1,
  },
  Stylise: {
    depthBands: 6,
    depthBandsFit: true,
    depthBandsNearColor: "#141028",
    depthBandsFarColor: "#c8d8ec",
    depthBandsFlatten: 1,
    posterize: 0,
    posterizeMode: "channel",
  },
  RenderSettings: {
    toneMapping: "aces",
  },
  ArtPalette: {
    rampSteps: 6,
  },
  ArtShape: {
    normals: "any",
  },
});

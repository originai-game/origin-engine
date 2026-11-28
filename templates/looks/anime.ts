import { file } from "@origingame/engine/author";

// Restored from anime-cel-3d before e1effb42f4. The art contract now also
// applies to side-view drawings; no camera projection is imposed on 3D.
export default file.look({
  name: "Anime",
  mood: "bright",
  ui: "@themes/anime",
  uiSeeds: {
    primary: "#4f5bd5",
    neutral: "#2a3060",
  },
  MaterialStyle: {
    toon: true,
    shadowTint: "#2a3060",
    shadowTintAmount: 0.7,
    outlineWidth: 2.5,
    outlineDistance: 10,
    outlineNearCap: 1,
    outlinePlace: "outside",
    outlineColor: "#141028",
  },
  RenderSettings: {
    toneMapping: "agx",
    autoExposure: true,
    adaptSpeedUp: 1,
    adaptSpeedDown: 3,
  },
  ArtPalette: {
    maxColors: 48,
    rampSteps: 2,
  },
  ArtTexels: {
    density: 128,
    maxSize: 2048,
  },
  ArtShape: {
    propFaces: 40000,
    characterFaces: 40000,
    normals: "smooth",
  },
  ArtLine: {
    width: 2.5,
    color: "#141028",
    distanceFalloff: 10,
  },
});

import { file } from "@origingame/engine/author";

export default file.look({
  // Flat facets in cel bands with hard edges (toon, toonSoftness 0) and six
  // clean tones, nothing else: no gradient survives on a surface. The facets
  // are the cook's (ArtShape normals flat), each a group of faces within 8
  // degrees with one normal and one colour, so a rolling ground reads as a
  // few broad planes, never as its triangles. The tones step value and tint
  // apart (posterizeMode tone), so a mood moves a colour's value, never its
  // hue.
  name: "Minimal geometric color",
  mood: "soft",
  ui: "@themes/minimal",
  // The UI's seed colours (UiSeeds), so the default style matches the
  // scene: one clean coral over plain grey.
  uiSeeds: {
    primary: "#ff6b4a",
    neutral: "#7a7a80",
  },
  MaterialStyle: {
    toon: true,
    toonSoftness: 0,
  },
  Stylise: {
    posterize: 6,
    posterizeMode: "tone",
  },
  RenderSettings: {
    toneMapping: "none",
  },
  // The art contract (docs/art.md §2.1), from the catalogue entry's palette and
  // assets: 3 to 6 colours; geometric primitives with flat faces.
  ArtPalette: {
    maxColors: 6,
  },
  ArtShape: {
    normals: "flat",
    facetAngle: 8,
  },
});

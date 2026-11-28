import { file } from "@origingame/engine/author";

export default file.look({
  // The same clean fills and warm contours on illustrated sprites and cel
  // shaded volumes. Comic crease accents survive; gritty hatching does not.
  name: "Cartoon",
  mood: "bright",
  ui: "@themes/cartoon",
  MaterialStyle: {
    toon: true,
    outlineWidth: 2,
    outlineDistance: 8,
    outlinePlace: "outside",
    outlineColor: "#2a1a10",
  },
  Stylise: {
    posterize: 10,
    posterizeMode: "tone",
  },
  PostEffects: {
    ao: false,
    vignette: true,
    vignetteIntensity: 0.2,
  },
  // The art contract (docs/art.md §2.1), from the catalogue entry's palette and
  // assets: 16 to 32 colours in ramps of three; a 2 px dark warm line round
  // every sprite; characters 96 to 160 px.
  ArtPalette: {
    maxColors: 32,
    rampSteps: 3,
  },
  ArtLine: {
    width: 2,
    color: "#2a1a10",
    place: "outside",
  },
  ArtShape: {
    propFaces: 30000,
    characterFaces: 30000,
    normals: "smooth",
  },
});

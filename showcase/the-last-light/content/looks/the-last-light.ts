import { file } from "@origingame/engine/author";
export default file.look({
  extends: ["faceted-low-poly"],
  mood: "warm",
  ui: "default",
  // The cover's grade: deeper shadows and shafts through the sea mist.
  // src/climate.ts drives the exposure and the colour temperature.
  RenderSettings: { bloomIntensity: 0.22 },
  PostEffects: {
    ao: true,
    aoIntensity: 1.2,
    contrast: 0.12,
    godRays: true,
    godRaysIntensity: 0.2,
  },
});

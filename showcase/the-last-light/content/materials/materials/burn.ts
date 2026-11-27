import { file, input } from "@origingame/engine/author";

// A charred rim, ember and white-hot edge, clipped to the element's shape.
export default file["ui-material"]({
  code: input.text("sources/materials/materials/burn.wgsl"),
  params: [
    { name: "progress", kind: "float", default: 0 }, // Burn progress, 0–1.
    { name: "edge", kind: "float", default: 6 }, // Rim width, CSS px.
    { name: "scale", kind: "float", default: 40 }, // Noise cell, CSS px.
    { name: "glow", kind: "color", default: [1, 0.36, 0.05] }, // Linear RGB.
  ],
});

import { file, input } from "@origingame/engine/author";

// A broad sheen over fine grain, driven by the clock or pointer tilt.
export default file["ui-material"]({
  code: input.text("sources/materials/materials/foil.wgsl"),
  params: [
    { name: "amount", kind: "float", default: 0.7 }, // Sheen, 0–1.
    { name: "grain", kind: "float", default: 1.5 }, // Grain cell, CSS px.
    { name: "sweep", kind: "vec2", default: [0.12, 0] }, // Sweeps/s, tilt offset.
    { name: "tint", kind: "color", default: [0.85, 0.92, 1] }, // Linear RGB.
  ],
});

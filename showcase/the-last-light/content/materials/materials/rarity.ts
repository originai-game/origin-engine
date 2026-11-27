import { file, input } from "@origingame/engine/author";

// A breathing tier-coloured edge glow; draw the frame and contents as usual.
export default file["ui-material"]({
  code: input.text("sources/materials/materials/rarity.wgsl"),
  params: [
    { name: "color", kind: "color", default: [1, 0.5, 0.05] }, // Linear RGB.
    { name: "size", kind: "float", default: 10 }, // Reach from edge, CSS px.
    { name: "pulse", kind: "float", default: 0.6 }, // Breaths/s, 0 steady.
    { name: "strength", kind: "float", default: 1 }, // Strength, 0–2.
  ],
});

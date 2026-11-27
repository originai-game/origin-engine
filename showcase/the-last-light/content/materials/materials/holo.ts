import { file, input } from "@origingame/engine/author";

// Fine iridescent stripes, weak enough that the element keeps its colour.
export default file["ui-material"]({
  code: input.text("sources/materials/materials/holo.wgsl"),
  params: [
    { name: "amount", kind: "float", default: 0.35 }, // Iridescence, 0–1.
    { name: "scale", kind: "float", default: 6 }, // Stripe period, CSS px.
    { name: "speed", kind: "float", default: 0.08 }, // Hue cycles/s.
    { name: "saturation", kind: "float", default: 0.9 }, // Saturation, 0–1.
  ],
});

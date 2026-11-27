import { file } from "@origingame/engine/author";

export default file.look({
  // Warm: golden light, amber highlights, slightly richer colour.
  RenderSettings: {
    exposure: 1.04,
  },
  PostEffects: {
    temperature: 0.3,
    tint: 0.03,
    saturation: 0.08,
    balanceHighlights: [0.04, 0, -0.04],
  },
});

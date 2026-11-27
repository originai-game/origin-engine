import { file } from "@origingame/engine/author";

export default file.look({
  // Cold: blue light, cooler shadows, slightly drained colour.
  RenderSettings: {
    exposure: 0.96,
  },
  PostEffects: {
    temperature: -0.3,
    saturation: -0.1,
    balanceShadows: [-0.03, 0, 0.05],
  },
});

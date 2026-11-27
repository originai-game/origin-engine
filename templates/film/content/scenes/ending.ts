import { file, ref } from "@origingame/engine/author";

// The live segment places this night stage. Its own sky provides ambient
// light/reflections instead of the look's daylight sky behind a dark backdrop.
export default file.scene({
  name: "Ending",
  resources: {
    "og.Sky": { enabled: true, zenith: [0.004, 0.006, 0.016], horizon: [0.02, 0.024, 0.036], ground: [0.006, 0.005, 0.004], sunSize: 0, sunIntensity: 0, glow: 0, stars: 0.8, moonSize: 1.2, moonDirection: [-0.42, 0.7, 0.56] },
  },
  entities: [
    { name: "set", model: ref("base/models/set") },
    { name: "moon", components: { "og.Transform": { position: [-3, 5, 4], quaternion: [-0.399593, -0.239756, 0, 0.884784] }, "og.DirectionalLight": { color: [0.55, 0.65, 0.9], intensity: 0.8, shadowSoftness: 0.02 } } },
    { name: "lamp-light", components: { "og.Transform": { position: [1, 0.95, 0.4] }, "og.PointLight": { color: [1, 0.72, 0.42], intensity: 2.5, distance: 8, radius: 0.25 } } },
  ],
});

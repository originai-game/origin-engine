import { file, ref } from "@origingame/engine/author";

// The in-engine cutscene's set: the camera track moves over it.
export default file.scene({
  name: "Hall",
  resources: {
    "og.Sky": { enabled: true, zenith: [0.1, 0.12, 0.25], horizon: [0.5, 0.4, 0.4], ground: [0.1, 0.08, 0.07], sunIntensity: 1.5, sunDirection: [-0.5, 0.4, 0.4] },
  },
  entities: [
    { name: "hall-set", model: ref("base/models/set") },
    { name: "hall-light", components: { "og.Transform": { position: [-3, 5, 3], quaternion: [-0.399593, -0.239756, 0, 0.884784] }, "og.DirectionalLight": { color: [1, 0.8, 0.6], intensity: 2 } } },
  ],
});

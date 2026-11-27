import { file, ref } from "@origingame/engine/author";

// The 3D level: a ball on the set, which the game rolls off the edge.
export default file.scene({
  name: "Level",
  resources: {
    "og.Sky": { enabled: true, zenith: [0.25, 0.4, 0.7], horizon: [0.6, 0.7, 0.8], ground: [0.2, 0.18, 0.15], sunIntensity: 3, sunDirection: [0.4, 0.8, 0.3] },
  },
  entities: [
    { name: "set", model: ref("base/models/set"), components: { "og.RigidBody": { kind: "fixed" }, "og.Collider": { shape: "box", size: [12, 0.1, 12] } } },
    { name: "ball", model: ref("base/models/ball"), components: { "og.Transform": { position: [0, 0.3, 0] }, "og.RigidBody": { mass: 1 }, "og.Collider": { shape: "sphere", radius: 0.3 } } },
    { name: "sun", components: { "og.Transform": { position: [3, 6, 2], quaternion: [-0.399593, 0.239756, 0, 0.884784] }, "og.DirectionalLight": { color: [1, 0.95, 0.85], intensity: 2.5 } } },
  ],
});

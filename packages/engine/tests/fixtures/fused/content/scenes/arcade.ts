import { file, ref } from "@origingame/engine/author";

// The xy-plane minigame switches the world to the 2D physics profile.
export default file.scene({
  name: "Arcade",
  resources: {
    "og.Physics": { profile: 2 },
    "og.Sky": { enabled: true, zenith: [0.05, 0.05, 0.1], horizon: [0.15, 0.1, 0.2], ground: [0.05, 0.05, 0.05], sunIntensity: 1, sunDirection: [0, 0.6, 0.8] },
  },
  entities: [
    { name: "arcade-floor", model: ref("base/models/set"), components: { "og.RigidBody": { kind: "fixed" }, "og.Collider": { shape: "box", size: [12, 0.1, 1] } } },
    { name: "puck", model: ref("base/models/ball"), components: { "og.Transform": { position: [0, 3, 0] }, "og.RigidBody": { mass: 1 }, "og.Collider": { shape: "sphere", radius: 0.3 } } },
    { name: "arcade-light", components: { "og.Transform": { position: [0, 5, 5], quaternion: [-0.3826834, 0, 0, 0.9238795] }, "og.DirectionalLight": { color: [0.8, 0.9, 1], intensity: 2 } } },
  ],
});

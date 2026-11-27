import { file } from '@origingame/engine/author';
// Effekseer node 0.7.0.4.4.0.0.0 "WindTracker"
export default file.effect({
  "main": {
    "texture": {
      "$ogRef": "base/vfx/effekseer-pierre02/textures/Particle_Hard"
    },
    "channel": "rgba",
    "uv": {
      "tile": [
        2.0,
        2.0
      ]
    }
  },
  "blend": "additive",
  "doubleSided": true
});

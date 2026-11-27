import { file } from '@origingame/engine/author';
// Effekseer node 4 "CenterFire"
export default file.effect({
  "main": {
    "texture": {
      "$ogRef": "base/vfx/effekseer-magicalxspiral/textures/ShockWave5"
    },
    "channel": "rgba",
    "uv": {
      "scroll": [
        0.0,
        0.9375
      ]
    }
  },
  "blend": "additive",
  "doubleSided": true
});

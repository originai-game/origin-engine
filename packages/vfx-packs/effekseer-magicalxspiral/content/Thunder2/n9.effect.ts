import { file } from '@origingame/engine/author';
// Effekseer node 9 "Impact1"
export default file.effect({
  "main": {
    "texture": {
      "$ogRef": "base/vfx/effekseer-magicalxspiral/textures/ShockWave1"
    },
    "channel": "rgba",
    "uv": {
      "tile": [
        16.0,
        1.0
      ]
    }
  },
  "blend": "additive",
  "doubleSided": true
});

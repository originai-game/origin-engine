import { file } from '@origingame/engine/author';
// Effekseer node 10 "BG"
export default file.effect({
  "main": {
    "texture": {
      "$ogRef": "base/vfx/effekseer-magicalxspiral/textures/ShockWave4"
    },
    "channel": "rgba",
    "uv": {
      "tile": [
        16.0,
        1.0
      ],
      "scroll": [
        1.875,
        0.0
      ]
    }
  },
  "blend": "additive",
  "doubleSided": true
});

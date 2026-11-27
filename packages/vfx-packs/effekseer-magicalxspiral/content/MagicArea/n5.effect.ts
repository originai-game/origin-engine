import { file } from '@origingame/engine/author';
// Effekseer node 5 "Node"
export default file.effect({
  "main": {
    "texture": {
      "$ogRef": "base/vfx/effekseer-magicalxspiral/textures/ShockWave4"
    },
    "channel": "rgba",
    "uv": {
      "tile": [
        4.0,
        1.0
      ]
    }
  },
  "blend": "additive",
  "doubleSided": true
});

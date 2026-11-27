import { file } from '@origingame/engine/author';
// Effekseer node 6 "Node"
export default file.effect({
  "main": {
    "texture": {
      "$ogRef": "base/vfx/effekseer-magicalxspiral/textures/ShockWave1"
    },
    "channel": "rgba",
    "uv": {
      "tile": [
        20.0,
        1.0
      ]
    }
  },
  "blend": "additive",
  "doubleSided": true
});

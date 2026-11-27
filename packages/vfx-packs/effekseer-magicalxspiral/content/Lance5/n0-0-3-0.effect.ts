import { file } from '@origingame/engine/author';
// Effekseer node 0.0.3.0 "Node"
export default file.effect({
  "main": {
    "texture": {
      "$ogRef": "base/vfx/effekseer-magicalxspiral/textures/Fire4"
    },
    "channel": "rgba",
    "uv": {
      "tile": [
        2.0,
        0.5
      ],
      "scroll": [
        0.0,
        0.4688
      ]
    }
  },
  "blend": "additive",
  "doubleSided": true
});

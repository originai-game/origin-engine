import { file } from '@origingame/engine/author';
// Effekseer node 1 "Node"
export default file.effect({
  "main": {
    "texture": {
      "$ogRef": "base/vfx/effekseer-magicalxspiral/textures/WaterForm1"
    },
    "channel": "rgba",
    "uv": {
      "scroll": [
        0.2344,
        -0.2344
      ]
    }
  },
  "alpha": {
    "kind": "curve",
    "keys": [
      [
        0.0,
        1.0
      ],
      [
        0.9722,
        1.0
      ],
      [
        1.0,
        0.0
      ]
    ]
  },
  "blend": "additive",
  "doubleSided": true
});

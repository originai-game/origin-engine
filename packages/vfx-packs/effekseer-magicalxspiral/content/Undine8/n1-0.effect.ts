import { file } from '@origingame/engine/author';
// Effekseer node 1.0 "Node"
export default file.effect({
  "main": {
    "texture": {
      "$ogRef": "base/vfx/effekseer-magicalxspiral/textures/Water"
    },
    "channel": "rgba",
    "uv": {
      "tile": [
        4.0,
        1.0
      ],
      "scroll": [
        1.875,
        0.0
      ]
    }
  },
  "alpha": {
    "kind": "curve",
    "keys": [
      [
        0.0,
        0.0
      ],
      [
        0.1,
        1.0
      ],
      [
        0.9,
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

import { file } from '@origingame/engine/author';
// Effekseer node 0 "Node"
export default file.effect({
  "main": {
    "texture": {
      "$ogRef": "base/vfx/effekseer-magicalxspiral/textures/Water"
    },
    "channel": "rgba",
    "uv": {
      "tile": [
        8.0,
        1.0
      ],
      "scroll": [
        0.0,
        -0.2344
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
        0.0364,
        1.0
      ],
      [
        1.0,
        1.0
      ]
    ]
  },
  "blend": "additive",
  "doubleSided": true
});

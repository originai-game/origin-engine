import { file } from '@origingame/engine/author';
// Effekseer node 1 "Arrow"
export default file.effect({
  "main": {
    "texture": {
      "$ogRef": "base/vfx/effekseer-effekseer01/textures/Arrow"
    },
    "channel": "rgba",
    "uv": {
      "scroll": [
        0.0,
        -0.9375
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
        0.0278,
        1.0
      ],
      [
        0.9444,
        1.0
      ],
      [
        1.0,
        0.0
      ]
    ]
  },
  "blend": "alpha",
  "doubleSided": true
});

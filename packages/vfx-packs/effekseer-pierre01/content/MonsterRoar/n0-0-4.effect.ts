import { file } from '@origingame/engine/author';
// Effekseer node 0.0.4 "Wind_Ring"
export default file.effect({
  "main": {
    "texture": {
      "$ogRef": "base/vfx/effekseer-pierre01/textures/Fire_Single"
    },
    "channel": "rgba",
    "uv": {}
  },
  "alpha": {
    "kind": "curve",
    "keys": [
      [
        0.0,
        0.0
      ],
      [
        0.2344,
        0.7185
      ],
      [
        0.4844,
        1.0
      ],
      [
        0.7188,
        0.8094
      ],
      [
        0.9677,
        0.125
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

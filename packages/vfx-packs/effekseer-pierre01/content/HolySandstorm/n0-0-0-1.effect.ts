import { file } from '@origingame/engine/author';
// Effekseer node 0.0.0.1 "Wind_Ring"
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
        0.0588,
        1.0
      ],
      [
        0.1765,
        1.0
      ],
      [
        0.7188,
        0.4049
      ],
      [
        0.875,
        0.1985
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

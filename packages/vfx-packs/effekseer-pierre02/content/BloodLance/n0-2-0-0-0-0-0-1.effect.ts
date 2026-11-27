import { file } from '@origingame/engine/author';
// Effekseer node 0.2.0.0.0.0.0.1 "Blood"
export default file.effect({
  "main": {
    "texture": {
      "$ogRef": "base/vfx/effekseer-pierre02/textures/Fire_Single"
    },
    "channel": "rgba",
    "uv": {
      "tile": [
        0.0,
        0.0
      ],
      "offset": [
        -0.4375,
        0.0
      ],
      "scroll": [
        0.2344,
        -0.1172
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
        0.8645,
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

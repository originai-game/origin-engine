import { file } from '@origingame/engine/author';
// Effekseer node 0.0.1.2 "Distortion"
export default file.effect({
  "main": {
    "texture": {
      "$ogRef": "base/vfx/effekseer-pierre01/textures/Fire"
    },
    "channel": "rgba",
    "uv": {
      "tile": [
        0.5,
        0.5
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
        0.2,
        1.0
      ],
      [
        0.4,
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

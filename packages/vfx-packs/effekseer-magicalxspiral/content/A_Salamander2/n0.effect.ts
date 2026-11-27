import { file } from '@origingame/engine/author';
// Effekseer node 0 "BG1"
export default file.effect({
  "main": {
    "texture": {
      "$ogRef": "base/vfx/effekseer-magicalxspiral/textures/ShockWave5"
    },
    "channel": "rgba",
    "uv": {
      "tile": [
        2.0,
        2.0
      ],
      "scroll": [
        0.0,
        0.9375
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
        0.2667,
        1.0
      ],
      [
        0.6667,
        1.0
      ],
      [
        1.0,
        0.0
      ]
    ]
  },
  "blend": "subtractive",
  "doubleSided": true
});

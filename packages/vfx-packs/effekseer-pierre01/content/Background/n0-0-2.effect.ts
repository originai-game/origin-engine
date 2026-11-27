import { file } from '@origingame/engine/author';
// Effekseer node 0.0.2 "Expander2"
export default file.effect({
  "main": {
    "texture": {
      "$ogRef": "base/vfx/effekseer-pierre01/textures/NoiseAnimation"
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
        0.25,
        0.7529
      ],
      [
        0.3438,
        0.9059
      ],
      [
        0.4375,
        0.9882
      ],
      [
        0.5312,
        1.0
      ],
      [
        0.625,
        0.9412
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

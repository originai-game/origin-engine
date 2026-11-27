import { file } from '@origingame/engine/author';
// Effekseer node 4 "Water"
export default file.effect({
  "main": {
    "texture": {
      "$ogRef": "base/vfx/effekseer-magicalxspiral/textures/WaterForm1"
    },
    "channel": "rgba",
    "uv": {
      "tile": [
        4.0,
        1.0
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
        0.2,
        0.6667
      ],
      [
        0.2969,
        0.8698
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
  "blend": "additive",
  "doubleSided": true
});

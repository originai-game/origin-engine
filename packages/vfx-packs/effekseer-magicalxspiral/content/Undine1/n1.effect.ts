import { file } from '@origingame/engine/author';
// Effekseer node 1 "WaterCylinder1"
export default file.effect({
  "main": {
    "texture": {
      "$ogRef": "base/vfx/effekseer-magicalxspiral/textures/WaterForm1"
    },
    "channel": "rgba",
    "uv": {
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
        0.025,
        1.0
      ],
      [
        0.875,
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

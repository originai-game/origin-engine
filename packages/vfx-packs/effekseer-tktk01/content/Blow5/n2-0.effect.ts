import { file } from '@origingame/engine/author';
// Effekseer node 2.0 "広がりリングギザギザ"
export default file.effect({
  "main": {
    "texture": {
      "$ogRef": "base/vfx/effekseer-tktk01/textures/Shock_wave_ring0011"
    },
    "channel": "rgba",
    "uv": {
      "tile": [
        2.0,
        1.0
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
        0.2031,
        0.0337
      ],
      [
        0.3125,
        0.1605
      ],
      [
        0.3594,
        0.2802
      ],
      [
        0.4062,
        0.4534
      ],
      [
        0.4531,
        0.6901
      ],
      [
        0.5,
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

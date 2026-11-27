import { file } from '@origingame/engine/author';
// Effekseer node 0.0.0.2.1 "Ring1"
export default file.effect({
  "main": {
    "texture": {
      "$ogRef": "base/vfx/effekseer-tktk02/textures/Shock_wave_ring0011"
    },
    "channel": "rgba",
    "uv": {
      "tile": [
        3.0,
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
        0.3333,
        1.0
      ],
      [
        0.5156,
        0.6358
      ],
      [
        0.6875,
        0.3482
      ],
      [
        0.8438,
        0.143
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

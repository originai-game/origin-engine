import { file } from '@origingame/engine/author';
// Effekseer node 5 "余韻ギザリング"
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
        1.0
      ],
      [
        0.8,
        1.0
      ],
      [
        0.8125,
        0.7874
      ],
      [
        0.8281,
        0.5756
      ],
      [
        0.8594,
        0.3021
      ],
      [
        0.8906,
        0.1766
      ],
      [
        0.9688,
        0.1
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

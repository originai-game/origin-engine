import { file } from '@origingame/engine/author';
// Effekseer node 0.0.0.0 "glow"
export default file.effect({
  "main": {
    "texture": {
      "$ogRef": "base/vfx/effekseer-tktk01/textures/Particle02"
    },
    "channel": "rgba",
    "uv": {
      "tile": [
        2.0,
        2.0
      ],
      "offset": [
        -0.5,
        -0.5
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
        0.0781,
        0.124
      ],
      [
        0.1094,
        0.4706
      ],
      [
        0.1333,
        1.0
      ],
      [
        0.8333,
        1.0
      ],
      [
        0.875,
        0.564
      ],
      [
        0.9062,
        0.3466
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

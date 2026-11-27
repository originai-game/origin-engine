import { file } from '@origingame/engine/author';
// Effekseer node 2.1 "Glow"
export default file.effect({
  "main": {
    "texture": {
      "$ogRef": "base/vfx/effekseer-tktk02/textures/Particle02"
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
        0.125,
        0.5356
      ],
      [
        0.25,
        1.0
      ],
      [
        0.4167,
        0.9052
      ],
      [
        0.5625,
        0.6167
      ],
      [
        0.7031,
        0.3777
      ],
      [
        0.8438,
        0.1774
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

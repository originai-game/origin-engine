import { file } from '@origingame/engine/author';
// Effekseer node 1.0.1 "Glow"
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
        0.0781,
        0.2941
      ],
      [
        0.1719,
        0.5987
      ],
      [
        0.3333,
        1.0
      ],
      [
        0.5,
        0.6252
      ],
      [
        0.6719,
        0.3259
      ],
      [
        0.8281,
        0.1305
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

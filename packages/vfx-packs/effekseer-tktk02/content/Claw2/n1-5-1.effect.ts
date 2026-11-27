import { file } from '@origingame/engine/author';
// Effekseer node 1.5.1 "Ring"
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
        0.1719,
        0.0674
      ],
      [
        0.25,
        0.1779
      ],
      [
        0.2969,
        0.3279
      ],
      [
        0.3281,
        0.4758
      ],
      [
        0.3594,
        0.6695
      ],
      [
        0.4,
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

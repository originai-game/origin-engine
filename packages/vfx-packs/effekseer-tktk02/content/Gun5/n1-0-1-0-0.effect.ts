import { file } from '@origingame/engine/author';
// Effekseer node 1.0.1.0.0 "shield"
export default file.effect({
  "main": {
    "texture": {
      "$ogRef": "base/vfx/effekseer-tktk02/textures/shield1"
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
        0.2031,
        0.2617
      ],
      [
        0.4062,
        1.0
      ],
      [
        0.5938,
        1.0
      ],
      [
        0.6094,
        0.9297
      ],
      [
        0.7031,
        0.2422
      ],
      [
        0.7969,
        0.0078
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

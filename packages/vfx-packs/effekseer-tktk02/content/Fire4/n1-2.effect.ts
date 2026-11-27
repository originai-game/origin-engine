import { file } from '@origingame/engine/author';
// Effekseer node 1.2 "Fire"
export default file.effect({
  "main": {
    "texture": {
      "$ogRef": "base/vfx/effekseer-tktk02/textures/Fire5"
    },
    "channel": "rgba",
    "uv": {
      "scroll": [
        0.0,
        0.5859
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
        0.25,
        1.0
      ],
      [
        0.75,
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

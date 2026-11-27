import { file } from '@origingame/engine/author';
// Effekseer node 2 "輪っか"
export default file.effect({
  "main": {
    "texture": {
      "$ogRef": "base/vfx/effekseer-nextsoft01/textures/Cloud01"
    },
    "channel": "rgba",
    "uv": {
      "scroll": [
        0.0,
        1.4062
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

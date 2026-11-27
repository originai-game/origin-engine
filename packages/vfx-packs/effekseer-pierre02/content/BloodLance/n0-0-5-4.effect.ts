import { file } from '@origingame/engine/author';
// Effekseer node 0.0.5.4 "Shockwave_End"
export default file.effect({
  "main": {
    "texture": {
      "$ogRef": "base/vfx/effekseer-pierre02/textures/Shockwave"
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
        0.5714,
        1.0
      ],
      [
        0.7188,
        0.8254
      ],
      [
        0.8571,
        0.5
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

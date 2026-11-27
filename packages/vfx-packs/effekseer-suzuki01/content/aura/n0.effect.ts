import { file } from '@origingame/engine/author';
// Effekseer node 0 "FlareMini"
export default file.effect({
  "main": {
    "texture": {
      "$ogRef": "base/vfx/effekseer-suzuki01/textures/aura3_type2"
    },
    "channel": "rgba",
    "uv": {
      "tile": [
        2.0,
        2.0
      ],
      "scroll": [
        0.5859,
        1.1719
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
        0.16,
        1.0
      ],
      [
        0.84,
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

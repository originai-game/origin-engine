import { file } from '@origingame/engine/author';
// Effekseer node 2 "Flare"
export default file.effect({
  "main": {
    "texture": {
      "$ogRef": "base/vfx/effekseer-suzuki01/textures/aura3_type2"
    },
    "channel": "rgba",
    "uv": {
      "scroll": [
        0.293,
        0.5859
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
        0.9,
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

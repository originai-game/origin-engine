import { file } from '@origingame/engine/author';
// Effekseer node 0.1 "Ring"
export default file.effect({
  "main": {
    "texture": {
      "$ogRef": "base/vfx/effekseer-magicalxspiral/textures/Line1"
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
        0.05,
        1.0
      ],
      [
        0.95,
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

import { file } from '@origingame/engine/author';
// Effekseer node 3 "Base"
export default file.effect({
  "main": {
    "texture": {
      "$ogRef": "base/vfx/effekseer-effekseer01/textures/Particle04_clear_hard"
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
        0.5,
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

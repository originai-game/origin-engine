import { file } from '@origingame/engine/author';
// Effekseer node 0.7 "柱中心glow"
export default file.effect({
  "main": {
    "texture": {
      "$ogRef": "base/vfx/effekseer-tktk01/textures/Particle02"
    },
    "channel": "rgba",
    "uv": {
      "tile": [
        1.0,
        0.5
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
        1.0,
        0.0
      ]
    ]
  },
  "blend": "additive",
  "doubleSided": true
});

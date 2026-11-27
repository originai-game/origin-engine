import { file } from '@origingame/engine/author';
// Effekseer node 0.2 "岩左"
export default file.effect({
  "main": {
    "texture": {
      "$ogRef": "base/vfx/effekseer-tktk01/textures/Hammer1"
    },
    "channel": "rgba",
    "uv": {
      "tile": [
        2.0,
        2.0
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
        0.6667,
        1.0
      ],
      [
        1.0,
        0.0
      ]
    ]
  },
  "blend": "alpha",
  "doubleSided": true
});

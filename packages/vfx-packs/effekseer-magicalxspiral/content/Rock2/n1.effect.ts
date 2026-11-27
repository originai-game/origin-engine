import { file } from '@origingame/engine/author';
// Effekseer node 1 "Rock"
export default file.effect({
  "main": {
    "texture": {
      "$ogRef": "base/vfx/effekseer-magicalxspiral/textures/Hammer1"
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
        0.2,
        1.0
      ],
      [
        1.0,
        1.0
      ]
    ]
  },
  "blend": "alpha",
  "doubleSided": true
});

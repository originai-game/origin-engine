import { file } from '@origingame/engine/author';
// Effekseer node 0.0.0 "Node"
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
        0.2667,
        1.0
      ],
      [
        0.9667,
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

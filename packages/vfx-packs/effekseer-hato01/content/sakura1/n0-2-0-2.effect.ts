import { file } from '@origingame/engine/author';
// Effekseer node 0.2.0.2 "Node"
export default file.effect({
  "main": {
    "texture": {
      "$ogRef": "base/vfx/effekseer-hato01/textures/sakura-1"
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
        0.1,
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
  "blend": "alpha",
  "doubleSided": true
});

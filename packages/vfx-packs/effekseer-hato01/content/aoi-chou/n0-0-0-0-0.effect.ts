import { file } from '@origingame/engine/author';
// Effekseer node 0.0.0.0.0 "Node"
export default file.effect({
  "main": {
    "texture": {
      "$ogRef": "base/vfx/effekseer-hato01/textures/aoi-hane"
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
        0.0833,
        1.0
      ],
      [
        0.9167,
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

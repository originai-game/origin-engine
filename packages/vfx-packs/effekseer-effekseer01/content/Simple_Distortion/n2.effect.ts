import { file } from '@origingame/engine/author';
// Effekseer node 2 "Distortion"
export default file.effect({
  "main": {
    "texture": {
      "$ogRef": "base/vfx/effekseer-effekseer01/textures/Cloud01"
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
        0.25,
        1.0
      ],
      [
        0.75,
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

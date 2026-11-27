import { file } from '@origingame/engine/author';
// Effekseer node 0.0 "時計リング"
export default file.effect({
  "main": {
    "texture": {
      "$ogRef": "base/vfx/effekseer-tktk01/textures/circle4"
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
        0.0118,
        1.0
      ],
      [
        0.5781,
        1.0
      ],
      [
        0.5938,
        0.9837
      ],
      [
        0.9062,
        0.0573
      ],
      [
        0.9219,
        0.0235
      ],
      [
        1.0,
        0.0235
      ]
    ]
  },
  "blend": "additive",
  "doubleSided": true
});

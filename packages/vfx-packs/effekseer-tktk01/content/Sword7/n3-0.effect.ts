import { file } from '@origingame/engine/author';
// Effekseer node 3.0 "〆リング"
export default file.effect({
  "main": {
    "texture": {
      "$ogRef": "base/vfx/effekseer-tktk01/textures/WaterForm1"
    },
    "channel": "rgba",
    "uv": {}
  },
  "alpha": {
    "kind": "curve",
    "keys": [
      [
        0.0,
        1.0
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

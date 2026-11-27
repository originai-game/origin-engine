import { file } from '@origingame/engine/author';
// Effekseer node 1 "WaterFall"
export default file.effect({
  "main": {
    "texture": {
      "$ogRef": "base/vfx/effekseer-magicalxspiral/textures/WaterForm1"
    },
    "channel": "rgba",
    "uv": {
      "scroll": [
        0.0,
        -0.9375
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

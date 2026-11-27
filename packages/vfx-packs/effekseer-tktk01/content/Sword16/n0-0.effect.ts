import { file } from '@origingame/engine/author';
// Effekseer node 0.0 "刃中心"
export default file.effect({
  "main": {
    "texture": {
      "$ogRef": "base/vfx/effekseer-tktk01/textures/ezzi"
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
        0.8333,
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

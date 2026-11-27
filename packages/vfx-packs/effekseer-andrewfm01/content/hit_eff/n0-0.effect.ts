import { file } from '@origingame/engine/author';
// Effekseer node 0.0 "Small Flashes"
export default file.effect({
  "main": {
    "texture": {
      "$ogRef": "base/vfx/effekseer-andrewfm01/textures/burst02_red"
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
        0.3333,
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

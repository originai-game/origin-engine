import { file } from '@origingame/engine/author';
// Effekseer node 4 "Shockwave"
export default file.effect({
  "main": {
    "texture": {
      "$ogRef": "base/vfx/effekseer-andrewfm01/textures/Shock_wave004"
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
        0.4667,
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

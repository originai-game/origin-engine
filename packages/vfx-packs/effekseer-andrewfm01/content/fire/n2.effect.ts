import { file } from '@origingame/engine/author';
// Effekseer node 2 "Shockwave"
export default file.effect({
  "main": {
    "texture": {
      "$ogRef": "base/vfx/effekseer-andrewfm01/textures/double-shockwave"
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
        0.25,
        1.0
      ],
      [
        0.3438,
        0.9351
      ],
      [
        0.4375,
        0.8186
      ],
      [
        0.75,
        0.2791
      ],
      [
        0.8438,
        0.1379
      ],
      [
        0.9219,
        0.0501
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

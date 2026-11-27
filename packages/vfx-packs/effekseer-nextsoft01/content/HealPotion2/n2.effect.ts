import { file } from '@origingame/engine/author';
// Effekseer node 2 "衝撃波"
export default file.effect({
  "main": {
    "texture": {
      "$ogRef": "base/vfx/effekseer-nextsoft01/textures/AuroraRing"
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
        0.5781,
        0.9
      ],
      [
        0.75,
        0.8415
      ],
      [
        0.8281,
        0.7482
      ],
      [
        0.875,
        0.6342
      ],
      [
        0.9219,
        0.4602
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

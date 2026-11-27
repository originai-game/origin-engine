import { file } from '@origingame/engine/author';
// Effekseer node 0.0.0.0 "刃上"
export default file.effect({
  "main": {
    "texture": {
      "$ogRef": "base/vfx/effekseer-tktk01/textures/Hammer1"
    },
    "channel": "rgba",
    "uv": {
      "tile": [
        4.0,
        4.0
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
        0.8594,
        0.958
      ],
      [
        0.9062,
        0.8395
      ],
      [
        0.9375,
        0.6685
      ],
      [
        0.9531,
        0.5466
      ],
      [
        0.9844,
        0.2153
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

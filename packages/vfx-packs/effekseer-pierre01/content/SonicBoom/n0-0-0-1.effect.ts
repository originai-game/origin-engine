import { file } from '@origingame/engine/author';
// Effekseer node 0.0.0.1 "Wind_RingPolygon"
export default file.effect({
  "main": {
    "texture": {
      "$ogRef": "base/vfx/effekseer-pierre01/textures/Fire"
    },
    "channel": "rgba",
    "uv": {
      "tile": [
        0.5,
        0.5
      ],
      "offset": [
        0.0,
        0.5
      ]
    }
  },
  "alpha": {
    "kind": "curve",
    "keys": [
      [
        0.0,
        0.0
      ],
      [
        0.0625,
        1.0
      ],
      [
        0.125,
        1.0
      ],
      [
        0.7031,
        0.4027
      ],
      [
        0.8594,
        0.2092
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

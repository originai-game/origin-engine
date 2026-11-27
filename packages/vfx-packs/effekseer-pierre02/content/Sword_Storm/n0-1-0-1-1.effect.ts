import { file } from '@origingame/engine/author';
// Effekseer node 0.1.0.1.1 "Wind_RingPolygon"
export default file.effect({
  "main": {
    "texture": {
      "$ogRef": "base/vfx/effekseer-pierre02/textures/Fire"
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
        0.2571,
        1.0
      ],
      [
        0.2857,
        1.0
      ],
      [
        0.7656,
        0.3915
      ],
      [
        0.8906,
        0.2001
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

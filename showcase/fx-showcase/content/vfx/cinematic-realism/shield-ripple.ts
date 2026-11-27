import { file, ref } from '@origingame/engine/author';

export default file.effect({
  "alpha": {
    "kind": "curve",
    "keys": [
      [
        0,
        0.8999999761581421
      ],
      [
        0.30000001192092896,
        0.6000000238418579
      ],
      [
        1,
        0
      ]
    ]
  },
  "blend": "additive",
  "doubleSided": true,
  "erosion": {
    "kind": "curve",
    "keys": [
      [
        0,
        0.20000000298023224
      ],
      [
        1,
        0.75
      ]
    ]
  },
  "noise": {
    "channel": "red",
    "uv": {
      "offset": [
        0,
        0
      ],
      "polar": false,
      "scroll": [
        0,
        0
      ],
      "tile": [
        24,
        1
      ],
      "rotate": 0
    }
  },
  "erosionSoftness": 0.20000000298023224,
  "erosionEdge": {
    "width": 0,
    "color": [
      1,
      1,
      1,
      1
    ]
  },
  "softDepth": 0.05000000074505806
});

import { file, ref } from '@origingame/engine/author';

export default file.effect({
  "alpha": {
    "kind": "curve",
    "keys": [
      [
        0,
        0.6000000238418579
      ],
      [
        0.4000000059604645,
        0.4000000059604645
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
        0.05000000074505806
      ],
      [
        0.5,
        0.30000001192092896
      ],
      [
        1,
        0.8500000238418579
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
        1,
        4
      ],
      "rotate": 0
    }
  },
  "erosionSoftness": 0.5,
  "erosionEdge": {
    "width": 0,
    "color": [
      1,
      1,
      1,
      1
    ]
  },
  "mask": {
    "texture": ref("base/vfx/cinematic-realism/textures/taper"),
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
        1,
        1
      ],
      "rotate": 0
    }
  },
  "softDepth": 0.05000000074505806
});

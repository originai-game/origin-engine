import { file, ref } from '@origingame/engine/author';

export default file.particles({
  "angularVelocity": [
    -8,
    8
  ],
  "blend": "alpha",
  "color": [
    0.15000000596046448,
    0.14000000059604645,
    0.14000000059604645,
    1
  ],
  "colorOverLife": [
    [
      0,
      [
        1,
        1,
        1,
        0
      ]
    ],
    [
      0.07999999821186066,
      [
        1,
        1,
        1,
        0.800000011920929
      ]
    ],
    [
      0.6000000238418579,
      [
        1,
        1,
        1,
        0.5
      ]
    ],
    [
      1,
      [
        1,
        1,
        1,
        0
      ]
    ]
  ],
  "drag": 0.4000000059604645,
  "duration": 2,
  "gravity": [
    0,
    0.5,
    0
  ],
  "lifetime": [
    6,
    9
  ],
  "lit": true,
  "maxParticles": 96,
  "noise": {
    "frequency": 0.30000001192092896,
    "scroll": 0.20000000298023224,
    "strength": 0.30000001192092896
  },
  "rate": 9,
  "rotation": [
    0,
    360
  ],
  "shape": {
    "kind": "sphere",
    "radius": 0.4000000059604645,
    "surface": false
  },
  "sheet": {
    "blend": true,
    "columns": 16,
    "frames": 64,
    "mode": {
      "kind": "life",
      "cycles": 1
    },
    "rows": 4
  },
  "size": [
    1.2000000476837158,
    1.7999999523162842
  ],
  "sizeOverLife": [
    [
      0,
      0.5
    ],
    [
      1,
      4
    ]
  ],
  "softDistance": 0.800000011920929,
  "speed": [
    0.6000000238418579,
    1.2000000476837158
  ],
  "texture": ref("base/vfx/cinematic-realism/textures/dust"),
  "wind": 1
});

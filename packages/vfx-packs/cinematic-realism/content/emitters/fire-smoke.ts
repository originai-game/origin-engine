import { file, ref } from '@origingame/engine/author';

export default file.particles({
  "angularVelocity": [
    -15,
    15
  ],
  "blend": "alpha",
  "color": [
    0.4000000059604645,
    0.38999998569488525,
    0.3799999952316284,
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
      0.15000000596046448,
      [
        1,
        1,
        1,
        0.2199999988079071
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
  "drag": 0.6000000238418579,
  "duration": 2,
  "gravity": [
    0,
    0.800000011920929,
    0
  ],
  "lifetime": [
    2.5,
    3.5
  ],
  "lit": true,
  "maxParticles": 24,
  "noise": {
    "frequency": 0.6000000238418579,
    "scroll": 0.30000001192092896,
    "strength": 0.20000000298023224
  },
  "rate": 5,
  "rotation": [
    0,
    360
  ],
  "shape": {
    "kind": "sphere",
    "radius": 0.10000000149011612,
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
    0.30000001192092896,
    0.44999998807907104
  ],
  "sizeOverLife": [
    [
      0,
      0.6000000238418579
    ],
    [
      1,
      2.799999952316284
    ]
  ],
  "softDistance": 0.20000000298023224,
  "speed": [
    0.30000001192092896,
    0.6000000238418579
  ],
  "texture": ref("base/vfx/cinematic-realism/textures/dust"),
  "wind": 1
});

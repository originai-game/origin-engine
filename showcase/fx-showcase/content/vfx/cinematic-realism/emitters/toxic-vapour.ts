import { file, ref } from '@origingame/engine/author';

export default file.particles({
  "angularVelocity": [
    -12,
    12
  ],
  "blend": "alpha",
  "color": [
    0.6000000238418579,
    0.7599999904632568,
    0.2199999988079071,
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
      0.25,
      [
        1,
        1,
        1,
        0.30000001192092896
      ]
    ],
    [
      0.6000000238418579,
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
  "drag": 1,
  "duration": 2,
  "gravity": [
    0,
    0.25,
    0
  ],
  "lifetime": [
    1.7999999523162842,
    2.5999999046325684
  ],
  "maxParticles": 44,
  "noise": {
    "frequency": 0.6000000238418579,
    "scroll": 0.30000001192092896,
    "strength": 0.20000000298023224
  },
  "rate": 14,
  "rotation": [
    0,
    360
  ],
  "shape": {
    "kind": "box",
    "halfExtents": [
      0.25,
      0.5,
      0.20000000298023224
    ]
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
    0.4000000059604645,
    0.6000000238418579
  ],
  "sizeOverLife": [
    [
      0,
      0.6000000238418579
    ],
    [
      1,
      2.4000000953674316
    ]
  ],
  "softDistance": 0.30000001192092896,
  "speed": [
    0.05000000074505806,
    0.25
  ],
  "texture": ref("base/vfx/cinematic-realism/textures/dust"),
  "wind": 1
});

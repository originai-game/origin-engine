import { file, ref } from '@origingame/engine/author';

export default file.particles({
  "angularVelocity": [
    -3,
    3
  ],
  "blend": "alpha",
  "color": [
    1.2999999523162842,
    1.340000033378601,
    1.399999976158142,
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
      0.20000000298023224,
      [
        1,
        1,
        1,
        0.07500000298023224
      ]
    ],
    [
      0.800000011920929,
      [
        1,
        1,
        1,
        0.07500000298023224
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
  "drag": 0.5,
  "duration": 2,
  "gravity": [
    0,
    0,
    0
  ],
  "lifetime": [
    8,
    12
  ],
  "lit": true,
  "maxParticles": 140,
  "rate": 11,
  "rotation": [
    0,
    360
  ],
  "shape": {
    "kind": "box",
    "halfExtents": [
      3,
      0.10000000149011612,
      3
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
    3.200000047683716,
    4.800000190734863
  ],
  "softDistance": 1,
  "speed": [
    0.05000000074505806,
    0.20000000298023224
  ],
  "spread": 90,
  "texture": ref("base/vfx/cinematic-realism/textures/dust"),
  "wind": 1
});

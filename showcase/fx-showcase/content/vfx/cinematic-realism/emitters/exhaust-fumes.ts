import { file, ref } from '@origingame/engine/author';

export default file.particles({
  "blend": "alpha",
  "color": [
    0.949999988079071,
    0.949999988079071,
    0.9800000190734863,
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
        0.20000000298023224
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
  "drag": 2,
  "duration": 2,
  "gravity": [
    0,
    0.25,
    0
  ],
  "lifetime": [
    1,
    1.7999999523162842
  ],
  "lit": true,
  "maxParticles": 48,
  "rate": 20,
  "rotation": [
    0,
    360
  ],
  "shape": {
    "kind": "cone",
    "angleDegrees": 12,
    "radius": 0.019999999552965164
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
    0.10000000149011612,
    0.1599999964237213
  ],
  "sizeOverLife": [
    [
      0,
      0.6000000238418579
    ],
    [
      1,
      4
    ]
  ],
  "softDistance": 0.10000000149011612,
  "speed": [
    0.6000000238418579,
    1.2000000476837158
  ],
  "texture": ref("base/vfx/cinematic-realism/textures/dust"),
  "wind": 1
});

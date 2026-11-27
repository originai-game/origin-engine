import { file, ref } from '@origingame/engine/author';

export default file.particles({
  "angularVelocity": [
    -10,
    10
  ],
  "blend": "alpha",
  "color": [
    0.949999988079071,
    1.0800000429153442,
    1.2000000476837158,
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
        0.30000001192092896
      ]
    ],
    [
      0.6000000238418579,
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
  "drag": 1.2000000476837158,
  "duration": 2,
  "gravity": [
    0,
    0.10000000149011612,
    0
  ],
  "lifetime": [
    2,
    3.5
  ],
  "maxParticles": 48,
  "rate": 14,
  "rotation": [
    0,
    360
  ],
  "shape": {
    "kind": "box",
    "halfExtents": [
      1,
      0.20000000298023224,
      0.4000000059604645
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
    1,
    1.600000023841858
  ],
  "sizeOverLife": [
    [
      0,
      0.5
    ],
    [
      1,
      2.200000047683716
    ]
  ],
  "softDistance": 0.5,
  "speed": [
    0.6000000238418579,
    1.600000023841858
  ],
  "spread": 50,
  "texture": ref("base/vfx/cinematic-realism/textures/dust"),
  "wind": 1
});

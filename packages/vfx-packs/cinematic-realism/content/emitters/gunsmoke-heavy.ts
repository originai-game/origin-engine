import { file, ref } from '@origingame/engine/author';

export default file.particles({
  "angularVelocity": [
    -30,
    30
  ],
  "blend": "alpha",
  "bursts": [
    {
      "count": 10,
      "cycles": 1,
      "interval": 0,
      "time": 0
    }
  ],
  "color": [
    1,
    0.9800000190734863,
    0.949999988079071,
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
      0.10000000149011612,
      [
        1,
        1,
        1,
        0.4000000059604645
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
  "drag": 3,
  "duration": 0.05000000074505806,
  "gravity": [
    0,
    0.15000000596046448,
    0
  ],
  "lifetime": [
    1.2000000476837158,
    2
  ],
  "lit": true,
  "looping": false,
  "maxParticles": 12,
  "rate": 0,
  "rotation": [
    0,
    360
  ],
  "shape": {
    "kind": "cone",
    "angleDegrees": 22,
    "radius": 0.05000000074505806
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
    0.5
  ],
  "sizeOverLife": [
    [
      0,
      0.5
    ],
    [
      1,
      2.799999952316284
    ]
  ],
  "softDistance": 0.15000000596046448,
  "speed": [
    1.5,
    4
  ],
  "texture": ref("base/vfx/cinematic-realism/textures/dust"),
  "wind": 1
});

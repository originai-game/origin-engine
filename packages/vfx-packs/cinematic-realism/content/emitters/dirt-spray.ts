import { file, ref } from '@origingame/engine/author';

export default file.particles({
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
    0.44999998807907104,
    0.36000001430511475,
    0.25999999046325684,
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
        0.699999988079071
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
    -1.5,
    0
  ],
  "lifetime": [
    0.699999988079071,
    1.2000000476837158
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
    "radius": 0.03999999910593033
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
    0.1599999964237213,
    0.30000001192092896
  ],
  "sizeOverLife": [
    [
      0,
      0.5
    ],
    [
      1,
      2.5999999046325684
    ]
  ],
  "softDistance": 0.11999999731779099,
  "speed": [
    0.4000000059604645,
    4.5
  ],
  "texture": ref("base/vfx/cinematic-realism/textures/dust"),
  "wind": 1
});

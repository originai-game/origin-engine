import { file, ref } from '@origingame/engine/author';

export default file.particles({
  "blend": "alpha",
  "bursts": [
    {
      "count": 7,
      "cycles": 1,
      "interval": 0,
      "time": 0
    }
  ],
  "color": [
    1,
    0.9200000166893005,
    0.7799999713897705,
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
        0.41999998688697815
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
    0.05000000074505806,
    0
  ],
  "lifetime": [
    0.699999988079071,
    1.2000000476837158
  ],
  "lit": true,
  "looping": false,
  "maxParticles": 8,
  "rate": 0,
  "rotation": [
    0,
    360
  ],
  "shape": {
    "kind": "cone",
    "angleDegrees": 75,
    "radius": 0.05999999865889549
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
    0.25999999046325684
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
  "softDistance": 0.10000000149011612,
  "speed": [
    0.4000000059604645,
    1
  ],
  "texture": ref("base/vfx/cinematic-realism/textures/dust"),
  "wind": 1
});

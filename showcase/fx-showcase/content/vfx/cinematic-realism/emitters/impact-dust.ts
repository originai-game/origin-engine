import { file, ref } from '@origingame/engine/author';

export default file.particles({
  "blend": "alpha",
  "bursts": [
    {
      "count": 5,
      "cycles": 1,
      "interval": 0,
      "time": 0
    }
  ],
  "color": [
    0.8199999928474426,
    0.7900000214576721,
    0.7400000095367432,
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
        0.6000000238418579
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
    0.10000000149011612,
    0
  ],
  "lifetime": [
    0.6000000238418579,
    1.100000023841858
  ],
  "lit": true,
  "looping": false,
  "maxParticles": 6,
  "rate": 0,
  "rotation": [
    0,
    360
  ],
  "shape": {
    "kind": "cone",
    "angleDegrees": 35,
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
    0.14000000059604645,
    0.23999999463558197
  ],
  "sizeOverLife": [
    [
      0,
      0.5
    ],
    [
      1,
      2.4000000953674316
    ]
  ],
  "softDistance": 0.11999999731779099,
  "speed": [
    0.5,
    1.2000000476837158
  ],
  "texture": ref("base/vfx/cinematic-realism/textures/dust")
});
